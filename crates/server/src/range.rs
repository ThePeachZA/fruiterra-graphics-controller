//! Single-range HTTP `Range` parsing.
//!
//! Only one byte range is supported. Malformed headers are ignored (the full
//! representation is served), while syntactically valid but unsatisfiable
//! ranges produce `416 Range Not Satisfiable`.

/// Inclusive byte range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ByteRange {
    /// First byte, inclusive.
    pub start: u64,
    /// Last byte, inclusive.
    pub end: u64,
}

/// Result of parsing a `Range` header against a known representation length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeOutcome {
    /// No usable range; serve the full representation.
    Full,
    /// Serve only the requested range.
    Partial(ByteRange),
    /// The range cannot be satisfied; respond with `416`.
    Unsatisfiable,
}

/// Parses a single `bytes=` range for a representation of `len` bytes.
#[must_use]
pub fn parse_range(header: &str, len: u64) -> RangeOutcome {
    let Some(rest) = header.strip_prefix("bytes=") else {
        return RangeOutcome::Full;
    };
    let rest = rest.trim();
    if rest.is_empty() || rest.contains(',') {
        return RangeOutcome::Full;
    }
    let Some((start_text, end_text)) = rest.split_once('-') else {
        return RangeOutcome::Full;
    };

    if start_text.is_empty() {
        let Ok(suffix) = end_text.parse::<u64>() else {
            return RangeOutcome::Full;
        };
        if suffix == 0 || len == 0 {
            return RangeOutcome::Unsatisfiable;
        }
        let start = len.saturating_sub(suffix);
        return RangeOutcome::Partial(ByteRange {
            start,
            end: len - 1,
        });
    }

    let Ok(start) = start_text.parse::<u64>() else {
        return RangeOutcome::Full;
    };
    if start >= len {
        return RangeOutcome::Unsatisfiable;
    }
    let end = if end_text.is_empty() {
        len - 1
    } else {
        match end_text.parse::<u64>() {
            Ok(end) => end.min(len - 1),
            Err(_) => return RangeOutcome::Full,
        }
    };
    if end < start {
        return RangeOutcome::Unsatisfiable;
    }
    RangeOutcome::Partial(ByteRange { start, end })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_closed_and_open_ranges() {
        assert_eq!(
            parse_range("bytes=0-9", 100),
            RangeOutcome::Partial(ByteRange { start: 0, end: 9 })
        );
        assert_eq!(
            parse_range("bytes=10-", 100),
            RangeOutcome::Partial(ByteRange { start: 10, end: 99 })
        );
        assert_eq!(
            parse_range("bytes=-10", 100),
            RangeOutcome::Partial(ByteRange { start: 90, end: 99 })
        );
    }

    #[test]
    fn clamps_ends_past_the_representation() {
        assert_eq!(
            parse_range("bytes=90-200", 100),
            RangeOutcome::Partial(ByteRange { start: 90, end: 99 })
        );
    }

    #[test]
    fn ignores_malformed_or_multiple_ranges() {
        for header in [
            "items=0-9",
            "bytes=",
            "bytes=abc-def",
            "bytes=0-9,20-29",
            "bytes=0",
            "bytes=-",
        ] {
            assert_eq!(parse_range(header, 100), RangeOutcome::Full, "{header}");
        }
    }

    #[test]
    fn rejects_unsatisfiable_ranges() {
        assert_eq!(parse_range("bytes=100-", 100), RangeOutcome::Unsatisfiable);
        assert_eq!(parse_range("bytes=9-3", 100), RangeOutcome::Unsatisfiable);
        assert_eq!(parse_range("bytes=-0", 100), RangeOutcome::Unsatisfiable);
        assert_eq!(parse_range("bytes=0-", 0), RangeOutcome::Unsatisfiable);
    }
}
