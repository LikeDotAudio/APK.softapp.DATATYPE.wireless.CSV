// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The worker's self test: the runner answers `outgoing/selftest_run` with it.
//!
//! All `Depth::Own` — a worker has no hardware. The decisive check runs a real
//! job through [`crate::handle_job`] on a CSV compiled into the binary, so a
//! pass means the parser in THIS image reads a zone/group export.

use apkaudio_contracts::self_test::{Check, Depth, Report};

use crate::{CONFIG_INI, MODULE};

/// Five rows of the zone/group layout, one of them out of band (dropped).
const PROBE_CSV: &str = "ZONE,GROUP,DEVICE,NAME,FREQ\n\
A,G1,Shure - AD4Q - G57,Lead,566.600\n\
A,G1,Shure - AD4Q - G57,Spare,567.525\n\
B,,None,,568.425\n\
B,G2,Sennheiser,Host,568.950\n\
C,G3,Bogus,Out of band,42.0\n";

pub fn self_test(depth: Depth) -> Report {
    let mut report = Report::new(MODULE, depth);
    report.record(Check::from_result(
        "landing_spots_build",
        Depth::Own,
        "job, result and the self-test leaves address legal topics",
        landing_spots(),
    ));
    report.record(Check::from_result(
        "config_section_still_names_this_module",
        Depth::Own,
        "config.ini declares this module under the name the bus knows it by",
        config_declares_module(),
    ));
    report.record(Check::from_result(
        "a_job_round_trips_through_the_parser",
        Depth::Own,
        "a zone/group CSV job comes back ok with 4 in-band records",
        probe_job(),
    ));
    report
}

fn landing_spots() -> Result<(), String> {
    use apkaudio_contracts::{baremetal_io, self_test as st};
    baremetal_io::module_job(MODULE).map_err(|e| format!("{e:?}"))?;
    baremetal_io::module_result(MODULE).map_err(|e| format!("{e:?}"))?;
    st::run_topic(MODULE).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

fn config_declares_module() -> Result<(), String> {
    let wanted = format!("[{MODULE}]");
    if CONFIG_INI.lines().any(|line| line.trim() == wanted) {
        Ok(())
    } else {
        Err(format!("config.ini does not declare {wanted}"))
    }
}

fn probe_job() -> Result<(), String> {
    let job = serde_json::json!({"job_id": "selftest", "filename": "probe.csv", "text": PROBE_CSV});
    let out = crate::handle_job(job.to_string().as_bytes(), 1 << 16);
    match (out["ok"].as_bool(), out["result"]["total_records"].as_u64()) {
        (Some(true), Some(4)) => Ok(()),
        _ => Err(format!("unexpected answer: {out}")),
    }
}

pub fn self_test_json(depth_word: &str) -> String {
    apkaudio_contracts::self_test::run_as_json(MODULE, depth_word, self_test)
}

#[cfg(test)]
mod tests {
    use super::*;
    use apkaudio_contracts::self_test::Verdict;

    #[test]
    fn the_self_test_passes_against_this_build() {
        let report = self_test(Depth::Own);
        assert_eq!(report.verdict(), Verdict::Pass, "{}", report.headline());
    }
}
