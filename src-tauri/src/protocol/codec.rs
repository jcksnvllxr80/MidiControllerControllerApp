//! Framing for the wire protocol: newline-delimited JSON (one object per line,
//! human-debuggable over a serial monitor) with a correlation `id` per request.
//! Isolated here so the framing can be swapped (length-prefixed, COBS, CBOR)
//! without touching the transports or the UI.
//!
//! The read side is byte-by-byte over any `Read` (a `dyn SerialPort` trait
//! object can't satisfy a `BufRead` bound), so these helpers are generic and
//! testable against `std::io::Cursor`.

use std::io::{Read, Write};

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::{Request, Response};

/// Encode a request as a single newline-terminated JSON line:
/// `{"id": <n>, "op": "...", ...}\n`
pub fn encode_request(id: u64, req: &Request) -> Result<Vec<u8>> {
    let mut obj = serde_json::to_value(req)?;
    if let Value::Object(map) = &mut obj {
        map.insert("id".to_string(), Value::from(id));
    }
    let mut line = serde_json::to_vec(&obj)?;
    line.push(b'\n');
    Ok(line)
}

/// Interpret one received line. Returns:
/// - `Ok(Some(resp))` if it's the JSON frame matching `id`,
/// - `Ok(None)` for blank lines, device log/noise, or other ids (keep reading).
pub fn match_response_line(line: &str, id: u64) -> Result<Option<Response>> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let value: Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => return Ok(None), // skip device log / noise lines
    };
    match value.get("id").and_then(Value::as_u64) {
        Some(rid) if rid == id => Ok(Some(serde_json::from_value(value)?)),
        _ => Ok(None),
    }
}

/// Read one newline-terminated line, byte-by-byte (serial messages are small).
/// Strips a trailing `\r`. Errors on timeout or a closed stream.
pub fn read_line<R: Read + ?Sized>(reader: &mut R) -> Result<String> {
    let mut buf: Vec<u8> = Vec::with_capacity(128);
    let mut byte = [0u8; 1];
    loop {
        match reader.read(&mut byte) {
            Ok(0) => return Err(anyhow!("connection closed")),
            Ok(_) => match byte[0] {
                b'\n' => break,
                b'\r' => {}
                b => buf.push(b),
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                return Err(anyhow!("timed out waiting for response"));
            }
            Err(e) => return Err(anyhow!("read error: {}", e)),
        }
    }
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// One framed exchange over a stream: encode + write `req`, then read lines
/// until the frame with `id` arrives (skipping up to `max_skip` noise lines).
pub fn roundtrip<W: Write + ?Sized, R: Read + ?Sized>(
    writer: &mut W,
    reader: &mut R,
    id: u64,
    req: &Request,
    max_skip: usize,
) -> Result<Response> {
    let bytes = encode_request(id, req)?;
    writer.write_all(&bytes)?;
    writer.flush()?;

    for _ in 0..max_skip {
        let line = read_line(reader)?;
        if let Some(resp) = match_response_line(&line, id)? {
            return Ok(resp);
        }
    }
    Err(anyhow!("no matching response for request {}", id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn frame(id: u64, req: &Request) -> Value {
        let bytes = encode_request(id, req).unwrap();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.ends_with('\n'), "frame must be newline-terminated");
        serde_json::from_str(s.trim()).unwrap()
    }

    #[test]
    fn encode_unit_variant_has_op_and_id() {
        let v = frame(3, &Request::Identify);
        assert_eq!(v["op"], "identify");
        assert_eq!(v["id"], 3);
    }

    #[test]
    fn encode_struct_variant_keeps_fields() {
        let v = frame(7, &Request::GetSet { name: "Gig".into() });
        assert_eq!(v["op"], "get_set");
        assert_eq!(v["id"], 7);
        assert_eq!(v["name"], "Gig");
    }

    #[test]
    fn encode_write_variant_keeps_data() {
        let data = serde_json::json!({ "name": "X", "songs": [] });
        let v = frame(1, &Request::WriteSet { name: "X".into(), data });
        assert_eq!(v["op"], "write_set");
        assert_eq!(v["data"]["name"], "X");
    }

    #[test]
    fn match_skips_blank_noise_and_other_ids() {
        assert!(match_response_line("", 1).unwrap().is_none());
        assert!(match_response_line("   ", 1).unwrap().is_none());
        assert!(match_response_line("boot: hello", 1).unwrap().is_none());
        assert!(match_response_line(r#"{"id":2,"ok":true}"#, 1).unwrap().is_none());
    }

    #[test]
    fn match_returns_frame_for_matching_id() {
        let r = match_response_line(r#"{"id":1,"ok":true,"data":42}"#, 1)
            .unwrap()
            .unwrap();
        assert!(r.ok);
        assert_eq!(r.data.unwrap(), 42);
    }

    #[test]
    fn match_parses_error_response() {
        let r = match_response_line(r#"{"id":5,"ok":false,"error":"nope"}"#, 5)
            .unwrap()
            .unwrap();
        assert!(!r.ok);
        assert_eq!(r.error.unwrap(), "nope");
    }

    #[test]
    fn match_ignores_unknown_extra_fields() {
        let r = match_response_line(r#"{"id":1,"ok":true,"extra":"x","data":1}"#, 1)
            .unwrap()
            .unwrap();
        assert!(r.ok);
    }

    #[test]
    fn read_line_splits_and_strips_cr() {
        let mut c = Cursor::new(b"hello\r\nworld\n".to_vec());
        assert_eq!(read_line(&mut c).unwrap(), "hello");
        assert_eq!(read_line(&mut c).unwrap(), "world");
    }

    #[test]
    fn read_line_blank_then_data() {
        let mut c = Cursor::new(b"\npayload\n".to_vec());
        assert_eq!(read_line(&mut c).unwrap(), "");
        assert_eq!(read_line(&mut c).unwrap(), "payload");
    }

    #[test]
    fn read_line_eof_is_error() {
        let mut c = Cursor::new(Vec::<u8>::new());
        assert!(read_line(&mut c).is_err());
    }

    #[test]
    fn roundtrip_writes_request_and_finds_response() {
        let mut writer: Vec<u8> = Vec::new();
        let lines = "garbage\n{\"id\":99,\"ok\":true}\n{\"id\":1,\"ok\":true,\"data\":{\"x\":5}}\n";
        let mut reader = Cursor::new(lines.as_bytes().to_vec());

        let resp = roundtrip(&mut writer, &mut reader, 1, &Request::Ping, 16).unwrap();
        assert!(resp.ok);
        assert_eq!(resp.data.unwrap()["x"], 5);

        let sent: Value = serde_json::from_str(String::from_utf8(writer).unwrap().trim()).unwrap();
        assert_eq!(sent["id"], 1);
        assert_eq!(sent["op"], "ping");
    }

    #[test]
    fn roundtrip_errors_when_no_match_before_eof() {
        let mut writer: Vec<u8> = Vec::new();
        let mut reader = Cursor::new(b"{\"id\":2,\"ok\":true}\n".to_vec());
        assert!(roundtrip(&mut writer, &mut reader, 1, &Request::Ping, 16).is_err());
    }

    #[test]
    fn roundtrip_errors_when_exceeding_max_skip() {
        let mut writer: Vec<u8> = Vec::new();
        // Many wrong-id lines, never the right one within max_skip.
        let mut lines = String::new();
        for _ in 0..10 {
            lines.push_str("{\"id\":2,\"ok\":true}\n");
        }
        let mut reader = Cursor::new(lines.into_bytes());
        assert!(roundtrip(&mut writer, &mut reader, 1, &Request::Ping, 3).is_err());
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn encode_delete_variant() {
        let bytes = encode_request(4, &Request::DeletePedal { name: "P".into() }).unwrap();
        let v: Value = serde_json::from_str(String::from_utf8(bytes).unwrap().trim()).unwrap();
        assert_eq!(v["op"], "delete_pedal");
        assert_eq!(v["name"], "P");
        assert_eq!(v["id"], 4);
    }

    #[test]
    fn match_accepts_various_data_types() {
        for raw in [
            r#"{"id":1,"ok":true,"data":[1,2,3]}"#,
            r#"{"id":1,"ok":true,"data":"hi"}"#,
            r#"{"id":1,"ok":true,"data":true}"#,
            r#"{"id":1,"ok":true,"data":null}"#,
        ] {
            assert!(match_response_line(raw, 1).unwrap().unwrap().ok);
        }
    }

    #[test]
    fn match_handles_large_id() {
        let big = u64::MAX;
        let line = format!(r#"{{"id":{big},"ok":true}}"#);
        assert!(match_response_line(&line, big).unwrap().is_some());
        assert!(match_response_line(&line, big - 1).unwrap().is_none());
    }

    #[test]
    fn match_ignores_non_object_json() {
        assert!(match_response_line("[1,2,3]", 1).unwrap().is_none());
        assert!(match_response_line("42", 1).unwrap().is_none());
        assert!(match_response_line("\"a string\"", 1).unwrap().is_none());
    }

    #[test]
    fn match_tolerates_surrounding_whitespace() {
        let r = match_response_line("   {\"id\":1,\"ok\":true}   ", 1).unwrap();
        assert!(r.unwrap().ok);
    }

    #[test]
    fn match_missing_id_is_none() {
        assert!(match_response_line(r#"{"ok":true}"#, 1).unwrap().is_none());
    }

    #[test]
    fn read_line_without_trailing_newline_errors_at_eof() {
        let mut c = Cursor::new(b"no newline here".to_vec());
        assert!(read_line(&mut c).is_err());
    }

    #[test]
    fn read_line_handles_long_payloads() {
        let payload = "x".repeat(5000);
        let mut c = Cursor::new(format!("{payload}\n").into_bytes());
        assert_eq!(read_line(&mut c).unwrap().len(), 5000);
    }

    #[test]
    fn roundtrip_preserves_nested_data() {
        let mut w: Vec<u8> = Vec::new();
        let mut r = Cursor::new(b"{\"id\":1,\"ok\":true,\"data\":{\"a\":{\"b\":[1,2,{\"c\":3}]}}}\n".to_vec());
        let resp = roundtrip(&mut w, &mut r, 1, &Request::Ping, 8).unwrap();
        assert_eq!(resp.data.unwrap()["a"]["b"][2]["c"], 3);
    }

    #[test]
    fn roundtrip_matches_second_frame_when_first_is_wrong_id() {
        let mut w: Vec<u8> = Vec::new();
        let mut r = Cursor::new(b"{\"id\":7,\"ok\":false}\n{\"id\":3,\"ok\":true,\"data\":1}\n".to_vec());
        let resp = roundtrip(&mut w, &mut r, 3, &Request::Ping, 8).unwrap();
        assert!(resp.ok);
    }

    #[test]
    fn encode_is_a_single_line() {
        let bytes = encode_request(
            1,
            &Request::WriteSet { name: "n".into(), data: serde_json::json!({ "x": 1 }) },
        )
        .unwrap();
        assert_eq!(bytes.iter().filter(|&&b| b == b'\n').count(), 1);
        assert_eq!(*bytes.last().unwrap(), b'\n');
    }
}
