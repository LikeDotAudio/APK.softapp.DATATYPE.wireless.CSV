// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The wireless CSV importer as a worker (PLAN-3423.01).
//!
//! ```text
//!   browser ── APK.audio/System/Protocols/wireless_csv/outgoing/job ──▶ worker
//!   worker  ── APK.audio/System/Protocols/wireless_csv/incoming/result ─▶ browser
//! ```
//!
//! ONE JOB IN, ONE RESULT OUT, MATCHED BY `job_id`. Neither leaf retains (see
//! `contracts/rust/src/baremetal_io.rs`): a retained job would run again on
//! every reconnect, and a retained result would answer a question the next
//! reader never asked. A caller makes up its own `job_id` and listens for it.
//!
//! [`handle_job`] is the whole tool and has no bus in it, so the tests below
//! run it on the fixtures in `../test_data/` with no broker anywhere near.
//! [`run_job_agent`] hands it to the runner's job loop
//! (`apk_plugin_runner::jobs::JobWorker`); the envelope — `job_id`, `text` |
//! `content_b64`, the size limit, the versioned answer — is the wireless
//! kernel's (`kernel/src/job.rs`), shared with the IAS, SOUNDbase and Wireless
//! Workbench workers.

pub use apkaudio_wireless_csv as csv_importer;
pub mod carriers;
pub mod self_test;

use apkaudio_wireless_kernel::job;
use serde_json::Value;

/// The token the bus knows this worker by. The same word as `[wireless_csv]`.
pub const MODULE: &str = "wireless_csv";

/// The tool the result came from, for a reader that listens to several.
pub const TOOL: &str = "DATATYPE.wireless.CSV";

/// The compiled-in config, the same text the runner publishes at `/config`.
pub const CONFIG_INI: &str = include_str!("../config.ini");

/// Used when `config.ini` does not say: far beyond any real export.
const DEFAULT_MAX_JOB_BYTES: usize = 8 * 1024 * 1024;

/// `max_job_bytes` from `config.ini`, or the default.
pub fn max_job_bytes(config_ini: &str) -> usize {
    job::max_job_bytes(config_ini, MODULE, DEFAULT_MAX_JOB_BYTES)
}

/// Answer one job. Always returns a result document, never panics on input.
pub fn handle_job(payload: &[u8], max_bytes: usize) -> Value {
    match job::read_file_job(payload, TOOL, "upload.csv", "the CSV", max_bytes) {
        Ok(file) => file.answer(TOOL, csv_importer::classify_csv_str(&file.text(), &file.filename)),
        Err(failure) => failure,
    }
}

/// The carriers a CSV import names (ROAD-3519). One record is one carrier.
/// A CSV carries no serial, so the id is formed (src/carriers.rs) from
/// `source file | zone | group | device | channel name`, numbered `#n` in file
/// order where those repeat (an unnamed row is common). `channel` is the
/// file's channel name; the CSV has no channel number, model or maker, and
/// a CSV with no band column gets the ITU band of the frequency.
pub fn carriers_of(answer: &Value) -> Option<(String, Vec<carriers::Carrier>)> {
    if answer["ok"] != true {
        return None;
    }
    let result = &answer["result"];
    let file = text(result, "source_file").to_string();
    let list = result["records"].as_array()?.iter().filter_map(|r| {
        Some(carriers::Carrier {
            unit: carriers::Carrier::unit_of(&[&file, text(r, "zone"), text(r, "group"), text(r, "device"), text(r, "channel_name")]),
            name: text(r, "channel_name").into(),
            frequency_mhz: r["frequency_mhz"].as_f64()?,
            channel: text(r, "channel_name").into(),
            band: text(r, "band").into(),
            model: String::new(),
            manufacturer: String::new(),
            device: text(r, "device").into(),
            zone: text(r, "zone").into(),
            group: text(r, "group").into(),
            source_file: file.clone(),
        })
    });
    Some((file.clone(), carriers::number_units(list.collect())))
}

/// A text field of a result row; empty when absent.
fn text<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap_or("")
}

/// The carriers of every imported file, restated as a Device/ tree and
/// sighted for presence (src/carriers.rs). Blocking, beside the job loop.
pub fn run_device_agent(host: &str, port: u16) {
    carriers::run(MODULE, host, port, carriers::restate_s(CONFIG_INI, MODULE, 30));
}

/// The job loop: the runner's. Blocking; the runner starts it on its own
/// thread and exits the process if it ever returns, so Docker restarts it.
pub fn run_job_agent(host: &str, port: u16) {
    let max = max_job_bytes(CONFIG_INI);
    apk_plugin_runner::jobs::JobWorker::new(MODULE, TOOL, move |payload: &[u8]| {
        // One job taken off outgoing/job and run: this worker's packet (ROAD-3472).
        apk_plugin_runner::loopback::packet_received();
        let answer = handle_job(payload, max);
        // Its carriers on the bus as a Device/ tree (ROAD-3519).
        if let Some((file, list)) = carriers_of(&answer) {
            carriers::imported(&file, list);
        }
        answer
    })
        // A job is a file, and base64 grows it by a third.
        .max_packet(max / 3 * 4 + 64 * 1024)
        .describe(|a| format!("ok={} records={}", a["ok"], a["result"]["total_records"]))
        .run(host, port);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The manifest the binary compiles in parses, declares the loopback port
    /// and lists every key of config.ini's [wireless_csv] that code reads (ROAD-3472).
    #[test]
    fn the_compiled_in_manifest_parses_and_reads_every_config_key() {
        let manifest = apkaudio_contracts::embedded_manifest!().expect("agent.toml parses");
        assert_eq!(manifest.name, MODULE);
        assert_eq!(manifest.kind, "worker");
        assert_eq!(manifest.telemetry_port, 8224);
        let config = apk_plugin_runner::ini_to_json(CONFIG_INI);
        let keys: Vec<&str> = config[MODULE].as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(manifest.unread_config_keys(keys), Some(0), "a [wireless_csv] key nothing reads");
    }
    use apkaudio_contracts::baremetal_io;
    use base64::Engine as _;
    use serde_json::json;

    const VENUE: &str = include_str!("../../test_data/venue.csv");
    const SOUNDBASE: &str = include_str!("../../test_data/SB PDF.csv");

    fn job(body: Value) -> Vec<u8> {
        body.to_string().into_bytes()
    }

    #[test]
    fn a_text_job_is_parsed_and_answered_under_its_id() {
        let out = handle_job(&job(json!({"job_id": "j1", "filename": "venue.csv", "text": VENUE})), 1 << 20);
        assert_eq!(out["job_id"], "j1");
        assert_eq!(out["ok"], true, "{out}");
        assert_eq!(out["result"]["pattern"], "StandardZoneGroup");
        // The count the README records for venue.csv, read straight from the file.
        assert_eq!(out["result"]["total_records"], 253);
        assert_eq!(out["result"]["source_file"], "venue.csv");
        assert_eq!(out["schema"], job::SCHEMA);
    }

    /// The wire shape a browser reads. A parser field renamed is a failure
    /// here first; change it on purpose, together with `kernel::job::SCHEMA`.
    #[test]
    fn the_result_shape_is_pinned() {
        let out = handle_job(&job(json!({"job_id": "s", "filename": "venue.csv", "text": VENUE})), 1 << 20);
        let keys = |v: &Value| {
            let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
            k.sort();
            k
        };
        assert_eq!(
            keys(&out["result"]),
            ["pattern", "pattern_name", "records", "skipped_rows", "source_file", "status", "total_records"]
        );
        assert_eq!(keys(&out["result"]["records"][0]), ["band", "channel_name", "device", "frequency_mhz", "group", "zone"]);
        assert_eq!(job::SCHEMA, 1);
    }

    #[test]
    fn a_base64_job_is_the_same_as_its_text() {
        let b64 = base64::engine::general_purpose::STANDARD.encode(SOUNDBASE);
        let out = handle_job(&job(json!({"job_id": "j2", "filename": "SB PDF.csv", "content_b64": b64})), 1 << 20);
        assert_eq!(out["ok"], true, "{out}");
        assert_eq!(out["result"]["pattern"], "SoundbaseReport");
        assert_eq!(out["result"]["total_records"], 20);
        assert_eq!(out["result"]["records"][0]["band"], "G57 - Narrowband");
    }

    #[test]
    fn a_bad_job_is_answered_not_dropped() {
        let out = handle_job(b"not json", 1 << 20);
        assert_eq!(out["ok"], false);
        assert!(out["job_id"].is_null());

        let out = handle_job(&job(json!({"text": VENUE})), 1 << 20);
        assert_eq!(out["ok"], false, "no job_id");

        let out = handle_job(&job(json!({"job_id": "j3"})), 1 << 20);
        assert_eq!(out["job_id"], "j3");
        assert_eq!(out["ok"], false, "no content");

        let out = handle_job(&job(json!({"job_id": "j4", "content_b64": "%%%"})), 1 << 20);
        assert_eq!(out["ok"], false, "bad base64");
    }

    #[test]
    fn an_oversized_job_is_refused_by_size() {
        let out = handle_job(&job(json!({"job_id": "j5", "text": VENUE})), 100);
        assert_eq!(out["ok"], false);
        assert!(out["error"].as_str().unwrap().contains("at most 100"));
    }

    #[test]
    fn the_limit_comes_from_config_ini() {
        assert_eq!(max_job_bytes(CONFIG_INI), 8_388_608);
        assert_eq!(max_job_bytes("[wireless_csv]\n"), DEFAULT_MAX_JOB_BYTES);
    }

    #[test]
    fn job_and_result_topics_are_this_modules() {
        let (job, retain) = baremetal_io::module_job(MODULE).unwrap();
        assert_eq!(job, "APK.audio/System/Protocols/wireless_csv/outgoing/job");
        assert!(!retain);
        let (result, retain) = baremetal_io::module_result(MODULE).unwrap();
        assert_eq!(result, "APK.audio/System/Protocols/wireless_csv/incoming/result");
        assert!(!retain);
    }
    /// ROAD-3519: every carrier of the fixture becomes a Device/ row with
    /// PLUGIN.WIFI's leaf names, under an id formed from the file, not an address.
    #[test]
    fn the_fixture_puts_every_carrier_under_device() {
        let out = handle_job(&job(json!({"job_id": "d", "filename": "venue.csv", "text": VENUE})), 1 << 24);
        assert_eq!(out["ok"], true, "{out}");
        let (file, list) = carriers_of(&out).expect("an ok answer has carriers");
        assert_eq!(file, "venue.csv");
        assert_eq!(list.len() as u64, out["result"]["total_records"].as_u64().unwrap());
        assert!(!list.is_empty());
        let ids: std::collections::BTreeSet<String> = list.iter().map(|c| c.device_id(MODULE)).collect();
        assert_eq!(ids.len(), list.len(), "every carrier its own id");
        let mut bands = 0;
        for c in &list {
            let f = c.fields(MODULE);
            let get = |leaf: &str| f.iter().find(|(t, _)| t.ends_with(&format!("/{leaf}"))).map(|(_, v)| v.clone()).unwrap();
            assert!(f[0].0.starts_with("APK.audio/System/Protocols/wireless_csv/Device/wireless_csv_plan_"), "{}", f[0].0);
            assert_eq!(get("device_class"), "Wireless Transmitter");
            let mhz: f64 = get("frequency_mhz").parse().unwrap();
            assert!(mhz > 100.0 && mhz < 3000.0, "{mhz}");
            assert!(!get("channel").is_empty() && !get("name").is_empty());
            if get("band") != "-" {
                bands += 1;
            }
            assert!(c.device_id(MODULE).starts_with("wireless_csv:plan:"));
        }
        assert!(bands > 0, "the band leaf is said (the file's, else the ITU band)");
        // The same file again lands on the same ids.
        let (_, again) = carriers_of(&handle_job(&job(json!({"job_id": "d", "filename": "venue.csv", "text": VENUE})), 1 << 24)).unwrap();
        assert_eq!(again.iter().map(|c| c.device_id(MODULE)).collect::<Vec<_>>(), list.iter().map(|c| c.device_id(MODULE)).collect::<Vec<_>>());
        assert!(carriers_of(&handle_job(b"not json", 1 << 24)).is_none(), "a failed job names no carrier");
    }
}
