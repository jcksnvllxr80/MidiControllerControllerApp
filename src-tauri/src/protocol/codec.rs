//! Framing for the wire protocol: newline-delimited JSON (one object per line,
//! human-debuggable over a serial monitor) with a correlation `id` per request.
//! Isolated here so the framing can be swapped (length-prefixed, COBS, CBOR)
//! without touching the transports or the UI.
//!
//! Reading is done byte-by-byte by the transport (a `dyn SerialPort` trait
//! object can't satisfy a `Read`/`BufRead` generic bound), so this module only
//! encodes requests and matches received lines.

use anyhow::Result;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_includes_id_and_op() {
        let bytes = encode_request(7, &Request::GetSet { name: "Gig".into() }).unwrap();
        let line = String::from_utf8(bytes).unwrap();
        assert!(line.ends_with('\n'));
        let v: Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["id"], 7);
        assert_eq!(v["op"], "get_set");
        assert_eq!(v["name"], "Gig");
    }

    #[test]
    fn matches_only_its_own_id() {
        assert!(match_response_line("not json", 1).unwrap().is_none());
        assert!(match_response_line(r#"{"id":2,"ok":true}"#, 1).unwrap().is_none());
        let r = match_response_line(r#"{"id":1,"ok":true,"data":42}"#, 1)
            .unwrap()
            .unwrap();
        assert!(r.ok);
        assert_eq!(r.data.unwrap(), 42);
    }
}
