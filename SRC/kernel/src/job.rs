// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The wireless file job, read once and answered once.
//!
//! ```text
//!   job     {"job_id": "<caller's>", "filename": "venue.csv", "text": "<the file>"}
//!           ("content_b64" in place of "text" for raw bytes)
//!   result  {"schema": 1, "job_id", "tool", "ok", "result", "error", "elapsed_ms"}
//! ```
//!
//! THE RESULT IS VERSIONED. `schema` is [`SCHEMA`]; it rises whenever the
//! document or a worker's `result` changes a key a reader could rely on. Each
//! worker pins its `result` keys with a shape test, so a renamed parser field
//! fails that test before it reaches a browser. The busy answer a worker sends
//! when its queue is full is the runner's shared shape (`apk_plugin_runner::jobs`)
//! and carries no `schema`: it never carries a `result` either.
//!
//! THE LOOP IS THE RUNNER'S (`apk_plugin_runner::jobs::JobWorker`). This module
//! is only what sits between a job payload and the parser: decode, limit, wrap.

use std::time::Instant;

use base64::Engine as _;
use serde::Serialize;
use serde_json::{json, Value};

/// The version of the wireless result document.
pub const SCHEMA: u64 = 1;

/// One decoded file job.
pub struct FileJob {
    pub job_id: Value,
    pub filename: String,
    /// The file as posted: `text` as UTF-8 bytes, or `content_b64` decoded.
    pub bytes: Vec<u8>,
    started: Instant,
}

impl FileJob {
    /// The file as text. Lossy rather than refused: an export saved as
    /// Windows-1252 still carries its frequencies in ASCII.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.bytes).into_owned()
    }

    /// The answer to this job: `ok: true` with `result`, or the failure.
    pub fn answer<T: Serialize>(&self, tool: &str, parsed: Result<T, String>) -> Value {
        match parsed.map(serde_json::to_value) {
            Ok(Ok(result)) => json!({
                "schema": SCHEMA,
                "job_id": self.job_id,
                "tool": tool,
                "ok": true,
                "result": result,
                "error": null,
                "elapsed_ms": elapsed_ms(self.started),
            }),
            Ok(Err(e)) => failure(tool, self.job_id.clone(), &format!("the result does not serialise: {e}"), self.started),
            Err(e) => failure(tool, self.job_id.clone(), &e, self.started),
        }
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

/// The failure document: the runner's `{job_id, tool, ok: false, result: null,
/// error, elapsed_ms}` plus `schema`.
pub fn failure(tool: &str, job_id: Value, error: &str, started: Instant) -> Value {
    let mut doc = apk_plugin_runner::jobs::failure(tool, job_id, error, elapsed_ms(started));
    doc["schema"] = json!(SCHEMA);
    doc
}

/// Read one job. `Err` is the finished failure document to send back: a
/// malformed job is answered, with its `job_id` when it had one, so the caller
/// waiting on that id is not left waiting forever.
///
/// `noun` names the file in the size message ("the CSV is … bytes").
pub fn read_file_job(
    payload: &[u8],
    tool: &str,
    default_filename: &str,
    noun: &str,
    max_bytes: usize,
) -> Result<FileJob, Value> {
    let started = Instant::now();
    let job: Value = match serde_json::from_slice(payload) {
        Ok(v @ Value::Object(_)) => v,
        Ok(_) => return Err(failure(tool, Value::Null, "a job is a JSON object", started)),
        Err(e) => return Err(failure(tool, Value::Null, &format!("the job is not JSON: {e}"), started)),
    };
    let job_id = match job.get("job_id") {
        Some(Value::String(s)) if !s.is_empty() => Value::String(s.clone()),
        _ => return Err(failure(tool, Value::Null, "the job has no job_id", started)),
    };
    let filename = job.get("filename").and_then(Value::as_str).unwrap_or(default_filename).to_string();
    let bytes = match (job.get("text"), job.get("content_b64")) {
        (Some(Value::String(t)), _) => t.clone().into_bytes(),
        (_, Some(Value::String(b))) => match base64::engine::general_purpose::STANDARD.decode(b.trim()) {
            Ok(bytes) => bytes,
            Err(e) => return Err(failure(tool, job_id, &format!("content_b64 does not decode: {e}"), started)),
        },
        _ => return Err(failure(tool, job_id, "the job carries neither text nor content_b64", started)),
    };
    if bytes.len() > max_bytes {
        let message = format!("{noun} is {} bytes; this worker takes at most {max_bytes}", bytes.len());
        return Err(failure(tool, job_id, &message, started));
    }
    Ok(FileJob { job_id, filename, bytes, started })
}

/// `[<module>] max_job_bytes` from a worker's `config.ini`, or `default`.
pub fn max_job_bytes(config_ini: &str, module: &str, default: usize) -> usize {
    apk_plugin_runner::ini_to_json(config_ini)
        .get(module)
        .and_then(|s| s.get("max_job_bytes"))
        .and_then(Value::as_str)
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOL: &str = "T";

    fn read(body: Value, max: usize) -> Result<FileJob, Value> {
        read_file_job(body.to_string().as_bytes(), TOOL, "upload.csv", "the CSV", max)
    }

    #[test]
    fn text_and_base64_read_the_same_bytes() {
        let a = read(json!({"job_id": "a", "text": "x,y"}), 100).ok().unwrap();
        let b64 = base64::engine::general_purpose::STANDARD.encode("x,y");
        let b = read(json!({"job_id": "b", "content_b64": b64, "filename": "f.csv"}), 100).ok().unwrap();
        assert_eq!(a.bytes, b.bytes);
        assert_eq!(a.filename, "upload.csv");
        assert_eq!(b.filename, "f.csv");
    }

    #[test]
    fn every_bad_job_is_answered_with_the_schema() {
        for (payload, has_id) in [
            (b"not json".to_vec(), false),
            (b"[1]".to_vec(), false),
            (json!({"text": "x"}).to_string().into_bytes(), false),
            (json!({"job_id": "j"}).to_string().into_bytes(), true),
            (json!({"job_id": "j", "content_b64": "%%%"}).to_string().into_bytes(), true),
        ] {
            let out = read_file_job(&payload, TOOL, "u", "the CSV", 100).err().unwrap();
            assert_eq!(out["ok"], false);
            assert_eq!(out["schema"], SCHEMA);
            assert_eq!(out["tool"], TOOL);
            assert_eq!(out["job_id"].is_string(), has_id, "{out}");
        }
    }

    #[test]
    fn an_oversized_job_names_its_size() {
        let out = read(json!({"job_id": "j", "text": "0123456789"}), 4).err().unwrap();
        assert_eq!(out["error"], "the CSV is 10 bytes; this worker takes at most 4");
    }

    #[test]
    fn the_answer_document_has_exactly_these_keys() {
        let job = read(json!({"job_id": "j", "text": "x"}), 100).ok().unwrap();
        let ok = job.answer(TOOL, Ok(json!({"n": 1})));
        let bad = job.answer::<Value>(TOOL, Err("nope".into()));
        for doc in [&ok, &bad] {
            let mut keys: Vec<_> = doc.as_object().unwrap().keys().cloned().collect();
            keys.sort();
            assert_eq!(keys, ["elapsed_ms", "error", "job_id", "ok", "result", "schema", "tool"]);
        }
        assert_eq!(ok["result"]["n"], 1);
        assert_eq!(bad["error"], "nope");
    }

    #[test]
    fn the_limit_reads_the_modules_section() {
        assert_eq!(max_job_bytes("[m]\nmax_job_bytes = 12\n", "m", 5), 12);
        assert_eq!(max_job_bytes("[m]\n", "m", 5), 5);
        assert_eq!(max_job_bytes("[other]\nmax_job_bytes = 12\n", "m", 5), 5);
    }
}
