//! End-to-end tests for the serial wire path **without hardware**.
//!
//! `FakeFirmware` is a duplex byte stream: the client writes newline-framed JSON
//! requests into it, and it answers with newline-framed JSON responses (computed
//! by the real `MockTransport` logic, framed with the matching correlation id).
//! Two handles — a writer and a reader sharing one buffer — model exactly how
//! `SerialTransport` opens a port and a `try_clone()` of it. Driving
//! `codec::roundtrip` through these handles exercises the full encode → wire →
//! decode → id-match path that real firmware will speak.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::rc::Rc;

use serde_json::Value;

use crate::protocol::{codec, Request, Response};
use crate::transport::{mock::MockTransport, Transport};

/// Shared state behind the writer/reader handle pair.
struct Shared {
    /// Bytes of the request line being accumulated (until `\n`).
    pending: Vec<u8>,
    /// Response bytes queued for the client to read.
    out: VecDeque<u8>,
    /// The "firmware" that actually computes answers.
    device: MockTransport,
    /// Prepend a non-JSON log line before each response (tests noise skipping).
    emit_noise: bool,
    /// Prepend a valid frame carrying the *previous* id (tests id correlation).
    emit_stale: bool,
}

impl Shared {
    /// Handle a complete request line: parse, answer via the mock, frame the
    /// response with the same id, and queue it (plus optional noise/stale).
    fn handle_line(&mut self, line: &[u8]) {
        let text = String::from_utf8_lossy(line);
        let value: Value = serde_json::from_str(text.trim()).expect("client sent valid JSON");
        let id = value["id"].as_u64().expect("framed request has id");
        let req: Request = serde_json::from_value(value).expect("framed request decodes");

        let resp = self.device.request(&req).expect("mock never errors at transport level");

        if self.emit_noise {
            self.queue_raw(b"boot: handling request\n");
        }
        if self.emit_stale && id > 0 {
            self.queue_response(id - 1, &Response::empty_ok());
        }
        self.queue_response(id, &resp);
    }

    fn queue_response(&mut self, id: u64, resp: &Response) {
        let mut v = serde_json::to_value(resp).unwrap();
        v.as_object_mut().unwrap().insert("id".into(), Value::from(id));
        let mut line = serde_json::to_vec(&v).unwrap();
        line.push(b'\n');
        self.queue_raw(&line);
    }

    fn queue_raw(&mut self, bytes: &[u8]) {
        self.out.extend(bytes.iter().copied());
    }
}

#[derive(Clone)]
struct WriteHandle {
    shared: Rc<RefCell<Shared>>,
}

#[derive(Clone)]
struct ReadHandle {
    shared: Rc<RefCell<Shared>>,
}

impl Write for WriteHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut s = self.shared.borrow_mut();
        for &b in buf {
            if b == b'\n' {
                let line = std::mem::take(&mut s.pending);
                s.handle_line(&line);
            } else {
                s.pending.push(b);
            }
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for ReadHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut s = self.shared.borrow_mut();
        let mut n = 0;
        while n < buf.len() {
            match s.out.pop_front() {
                Some(b) => {
                    buf[n] = b;
                    n += 1;
                }
                None => break,
            }
        }
        if n == 0 {
            // No data pending: model a serial read timeout rather than spin.
            return Err(io::Error::new(io::ErrorKind::TimedOut, "no data"));
        }
        Ok(n)
    }
}

/// A connected fake firmware, exposing a writer + reader like an open port.
struct FakeFirmware {
    writer: WriteHandle,
    reader: ReadHandle,
    next_id: u64,
}

impl FakeFirmware {
    fn new() -> Self {
        Self::with_options(false, false)
    }

    fn with_options(emit_noise: bool, emit_stale: bool) -> Self {
        let shared = Rc::new(RefCell::new(Shared {
            pending: Vec::new(),
            out: VecDeque::new(),
            device: MockTransport::new(),
            emit_noise,
            emit_stale,
        }));
        Self {
            writer: WriteHandle { shared: Rc::clone(&shared) },
            reader: ReadHandle { shared },
            next_id: 1,
        }
    }

    /// One full framed exchange, like `SerialTransport::request`.
    fn request(&mut self, req: &Request) -> Response {
        let id = self.next_id;
        self.next_id += 1;
        codec::roundtrip(&mut self.writer, &mut self.reader, id, req, 256, &mut |_| {}).unwrap()
    }
}

// ---- the tests --------------------------------------------------------------

#[test]
fn identify_handshake_over_the_wire() {
    let mut fw = FakeFirmware::new();
    let resp = fw.request(&Request::Identify);
    assert!(resp.ok);
    let data = resp.data.unwrap();
    assert_eq!(data["name"], "Mock MidiController");
    assert_eq!(data["protocol_version"], 1);
}

#[test]
fn ping_roundtrips() {
    let mut fw = FakeFirmware::new();
    assert!(fw.request(&Request::Ping).ok);
}

#[test]
fn list_then_get_a_set_over_the_wire() {
    let mut fw = FakeFirmware::new();
    let sets = fw.request(&Request::ListSets);
    let names = sets.data.unwrap();
    assert!(names.as_array().unwrap().iter().any(|v| v == "Friday Gig"));

    let set = fw.request(&Request::GetSet { name: "Friday Gig".into() });
    assert_eq!(set.data.unwrap()["songs"][0], "Intro");
}

#[test]
fn write_then_get_persists_over_the_wire() {
    let mut fw = FakeFirmware::new();
    fw.request(&Request::WriteSong {
        name: "Bridge".into(),
        data: serde_json::json!({ "name": "Bridge", "tempo": 88, "parts": {} }),
    });
    let got = fw.request(&Request::GetSong { name: "Bridge".into() });
    assert_eq!(got.data.unwrap()["tempo"], 88);
}

#[test]
fn delete_then_get_errors_over_the_wire() {
    let mut fw = FakeFirmware::new();
    fw.request(&Request::DeletePedal { name: "Timeline".into() });
    let got = fw.request(&Request::GetPedal { name: "Timeline".into() });
    assert!(!got.ok);
    assert!(got.error.unwrap().contains("Timeline"));
}

#[test]
fn control_ops_return_display_messages_over_the_wire() {
    let mut fw = FakeFirmware::new();
    let d = fw.request(&Request::Dpad { direction: "CW".into() });
    assert!(d.data.unwrap()["display_message"].as_str().unwrap().contains("CW"));
    let s = fw.request(&Request::Short { button: "2".into() });
    assert!(s.data.unwrap()["display_message"].as_str().unwrap().contains("2"));
}

#[test]
fn many_sequential_requests_keep_id_correlation() {
    let mut fw = FakeFirmware::new();
    // 50 interleaved requests; each must get its own answer back.
    for i in 0..50 {
        let resp = if i % 2 == 0 {
            fw.request(&Request::ListSongs)
        } else {
            fw.request(&Request::Ping)
        };
        assert!(resp.ok, "request {i} failed");
    }
}

#[test]
fn noisy_firmware_logs_are_skipped() {
    let mut fw = FakeFirmware::with_options(true, false);
    let resp = fw.request(&Request::Identify);
    assert!(resp.ok, "should skip the boot log line and find the response");
}

#[test]
fn stale_id_frames_are_skipped() {
    // The firmware prepends a valid frame carrying the previous id; the client
    // must ignore it and match only its own id.
    let mut fw = FakeFirmware::with_options(false, true);
    let one = fw.request(&Request::GetSet { name: "Friday Gig".into() });
    assert!(one.ok);
    assert_eq!(one.data.unwrap()["name"], "Friday Gig");
}

#[test]
fn noise_and_stale_together_still_resolve() {
    let mut fw = FakeFirmware::with_options(true, true);
    for _ in 0..10 {
        assert!(fw.request(&Request::Ping).ok);
    }
}

#[test]
fn full_session_walkthrough_over_the_wire() {
    // A realistic editor session: identify, browse, edit, verify, control.
    let mut fw = FakeFirmware::new();

    assert!(fw.request(&Request::Identify).ok);

    let sets = fw.request(&Request::ListSets).data.unwrap();
    assert_eq!(sets.as_array().unwrap().len(), 2);

    // Add a song to a set and save it.
    fw.request(&Request::WriteSet {
        name: "Friday Gig".into(),
        data: serde_json::json!({ "name": "Friday Gig", "songs": ["Intro", "Main Riff", "Ballad"] }),
    });
    let after = fw.request(&Request::GetSet { name: "Friday Gig".into() });
    assert_eq!(after.data.unwrap()["songs"].as_array().unwrap().len(), 3);

    // Create then remove a pedal.
    fw.request(&Request::WritePedal {
        name: "Compressor".into(),
        data: serde_json::json!({ "name": "Compressor", "presets": [0], "params": [] }),
    });
    assert!(fw.request(&Request::GetPedal { name: "Compressor".into() }).ok);
    fw.request(&Request::DeletePedal { name: "Compressor".into() });
    assert!(!fw.request(&Request::GetPedal { name: "Compressor".into() }).ok);

    // Live control.
    for dir in ["up", "down", "CW", "CCW"] {
        assert!(fw.request(&Request::Dpad { direction: dir.into() }).ok);
    }
    for btn in ["1", "2", "3", "4", "5"] {
        assert!(fw.request(&Request::Short { button: btn.into() }).ok);
    }
}

#[test]
fn every_request_variant_roundtrips_over_the_wire() {
    let mut fw = FakeFirmware::new();
    let data = serde_json::json!({});
    let reqs = vec![
        Request::Identify,
        Request::Ping,
        Request::ListSets,
        Request::GetSet { name: "Friday Gig".into() },
        Request::ListSongs,
        Request::GetSong { name: "Intro".into() },
        Request::ListPedals,
        Request::GetPedal { name: "Timeline".into() },
        Request::WriteSet { name: "w".into(), data: data.clone() },
        Request::WriteSong { name: "w".into(), data: data.clone() },
        Request::WritePart { name: "w".into(), data: data.clone() },
        Request::WritePedal { name: "w".into(), data: data.clone() },
        Request::DeleteSet { name: "w".into() },
        Request::DeleteSong { name: "w".into() },
        Request::DeletePart { name: "w".into() },
        Request::DeletePedal { name: "w".into() },
        Request::Dpad { direction: "up".into() },
        Request::Short { button: "1".into() },
    ];
    assert_eq!(reqs.len(), 18);
    for req in &reqs {
        // Every variant produces a well-formed framed response with no panic.
        let _ = fw.request(req);
    }
}
