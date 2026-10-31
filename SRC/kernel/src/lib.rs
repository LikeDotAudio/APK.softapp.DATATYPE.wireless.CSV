// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! What the four wireless importers (CSV, IAS, SOUNDbase, Shure Wireless
//! Workbench) and their workers agree on — one copy each.
//!
//! * [`usable_mhz`] / [`is_usable_mhz`] — a coordination frequency is usable
//!   when it lies strictly between 100 and 3000 MHz. Four importers used to
//!   re-type `f > 100.0 && f < 3000.0`; they now ask this function.
//! * [`import_status`] — `"success"` only when a file yielded records, else
//!   `"empty"`, never a success the panel would draw as a blank table.
//! * [`job`] (feature `worker`) — the file-job envelope and the result document.

/// The usable coordination band, exclusive at both ends, in MHz.
pub const USABLE_MHZ: (f64, f64) = (100.0, 3000.0);

/// True when `mhz` lies strictly inside [`USABLE_MHZ`].
pub fn is_usable_mhz(mhz: f64) -> bool {
    mhz > USABLE_MHZ.0 && mhz < USABLE_MHZ.1
}

/// `Some(mhz)` when it is usable, `None` otherwise — the shape the parsers filter with.
pub fn usable_mhz(mhz: f64) -> Option<f64> {
    is_usable_mhz(mhz).then_some(mhz)
}

/// The cell, parsed as MHz, when it is a usable frequency.
pub fn usable_mhz_text(text: &str) -> Option<f64> {
    text.trim().parse::<f64>().ok().and_then(usable_mhz)
}

/// The import's status word for `records` imported rows.
pub fn import_status(records: usize) -> String {
    if records > 0 { "success" } else { "empty" }.to_string()
}

#[cfg(feature = "worker")]
pub mod job;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_band_is_exclusive_at_both_ends() {
        assert!(!is_usable_mhz(100.0));
        assert!(is_usable_mhz(100.001));
        assert!(is_usable_mhz(2999.999));
        assert!(!is_usable_mhz(3000.0));
        assert!(!is_usable_mhz(42.0));
        assert!(!is_usable_mhz(f64::NAN));
    }

    #[test]
    fn a_cell_is_trimmed_and_parsed() {
        assert_eq!(usable_mhz_text(" 566.600 "), Some(566.6));
        assert_eq!(usable_mhz_text("No Match"), None);
        assert_eq!(usable_mhz_text("50"), None);
    }

    #[test]
    fn no_records_is_empty_not_success() {
        assert_eq!(import_status(0), "empty");
        assert_eq!(import_status(3), "success");
    }
}
