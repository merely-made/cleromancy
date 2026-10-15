// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Human chart entry converted once at the calculation boundary.

pub(super) fn instant(value: &str) -> String {
    let value = value.trim();
    if value.contains('T') {
        return value.to_string();
    }
    let value = value.replace(' ', "T");
    match value.len() {
        16 => format!("{value}:00Z"),
        19 => format!("{value}Z"),
        _ => value,
    }
}

/// Exact decimal conversion: no floating-point rounding at the input seam.
pub(super) fn degrees(value: &str, places: usize) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    let negative = value.starts_with('-');
    let unsigned = value.strip_prefix(['-', '+']).unwrap_or(value);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() > places
    {
        return Err(format!(
            "Enter degrees as a decimal with at most {places} decimal places."
        ));
    }
    let scale = 10_i64.pow(places as u32);
    let whole = whole.parse::<i64>().map_err(|_| "Angle is too large.")?;
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<i64>().map_err(|_| "Invalid angle.")?
            * 10_i64.pow((places - fraction.len()) as u32)
    };
    let scaled = whole
        .checked_mul(scale)
        .and_then(|v| v.checked_add(fraction))
        .ok_or("Angle is too large.")?;
    let scaled = if negative { -scaled } else { scaled };
    i32::try_from(scaled)
        .map(|v| v.to_string())
        .map_err(|_| "Angle is outside the supported range.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn human_entries_preserve_exact_calculation_units() {
        assert_eq!(instant("2026-10-15 09:30"), "2026-10-15T09:30:00Z");
        assert_eq!(degrees("-73.9857", 6).unwrap(), "-73985700");
        assert_eq!(degrees("1.25", 3).unwrap(), "1250");
        assert!(degrees("1.0001", 3).is_err());
        assert!(degrees("NaN", 6).is_err());
    }
}
