//! Human-only formatting. Native Numerica owns ordinary uncertainty rounding;
//! the adapter fixes the exponent policy and the large-error presentation.
use symbolica::numerical_integration::StatisticsAccumulator;

pub(crate) fn superscript(value: i32) -> String {
    spenso::utils::to_superscript(value as isize)
}

fn parts(value: f64, precision: Option<usize>) -> (f64, i32) {
    let s = precision.map_or_else(|| format!("{value:e}"), |p| format!("{value:.p$e}"));
    let (mantissa, exponent) = s.split_once('e').expect("finite scientific number");
    (
        mantissa.parse().expect("native mantissa"),
        exponent.parse().expect("native exponent"),
    )
}

fn short(value: f64) -> String {
    let s = format!("{value:.6}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

pub(crate) fn scientific(value: f64) -> String {
    if !value.is_finite() {
        return "unavailable".into();
    }
    let (mantissa, exponent) = parts(value.abs(), Some(6));
    let sign = if value < 0.0 { "−" } else { "+" };
    format!("{sign}{} ·10{}", short(mantissa), superscript(exponent))
}

/// Signed, normalized scientific notation. Missing error is explicitly unknown.
/// Parenthesized integer uncertainty always denotes units of the final displayed
/// central digit, including finite values spanning extreme exponent ranges.
pub(crate) fn uncertainty(value: f64, error: Option<f64>) -> String {
    if !value.is_finite() {
        return "unavailable".into();
    }
    let error = error.filter(|error| error.is_finite() && *error >= 0.0);
    let (mut mean, mut exponent) = if value != 0.0 {
        parts(value, None)
    } else if let Some(error) = error.filter(|error| *error > 0.0) {
        (0.0, parts(error, None).1)
    } else {
        (0.0, 0)
    };
    let sign = if value.is_sign_negative() && value != 0.0 {
        "−"
    } else {
        "+"
    };
    mean = mean.abs();
    let Some(error) = error else {
        return format!("{} (σ n/a)", scientific(value));
    };
    if error == 0.0 {
        return format!("{sign}{mean}(0) ·10{}", superscript(exponent));
    }
    // Rounding the error first handles decade carries without underflowing a
    // scale such as 10^-324. Both parts come from Rust's native float formatter.
    let (err_mantissa, err_exponent) = parts(error, Some(1));
    let mut shift = err_exponent - exponent;
    let mut decimals = (1 - shift).max(0) as usize;
    if decimals <= 15 {
        let factor = 10_f64.powi(decimals as i32);
        if (mean * factor).round() / factor >= 10.0 {
            mean /= 10.0;
            exponent += 1;
            shift -= 1;
            decimals = (1 - shift).max(0) as usize;
        }
    }
    let mantissa = if (-14..0).contains(&shift) {
        let mut native = StatisticsAccumulator::<f64>::new();
        native.avg = mean;
        native.err = err_mantissa * 10_f64.powi(shift);
        native.format_uncertainty()
    } else if shift == 0 {
        // Native Numerica uses absolute parenthesized error for this ratio.
        // Our fixed last-digit convention instead displays 1.2(56).
        format!("{mean:.1}({:.0})", err_mantissa * 10.0)
    } else if (1..=15).contains(&shift) {
        // At least one meaningful central digit remains when uncertainty is
        // larger than the value. Keep exactly two significant uncertainty digits.
        let central = format!("{mean:.decimals$}");
        let absolute_error = format!(
            "{:.*}",
            (1 - shift).max(0) as usize,
            err_mantissa * 10_f64.powi(shift)
        );
        format!("{central}({absolute_error})")
    } else if shift < 0 {
        // Native float formatting handles the entire finite f64 exponent range.
        // Preserve the last-digit convention even when it requires many digits.
        {
            let original = value.abs();
            let scientific = format!("{original:.decimals$e}");
            let exact_mantissa = scientific
                .split_once('e')
                .expect("finite scientific number")
                .0;
            format!("{exact_mantissa}({:.0})", err_mantissa * 10.0)
        }
    } else {
        // Construct only a presentation digit string; scaling the uncertainty
        // as an f64 here could overflow despite both original inputs being finite.
        let digits = format!("{:.0}", err_mantissa * 10.0);
        format!("{mean:.0}({digits}{})", "0".repeat((shift - 1) as usize))
    };
    format!("{sign}{mantissa} ·10{}", superscript(exponent))
}

/// Dashboard counts rounded to four significant digits, with B as the largest
/// unit. Widen before rounding so even u64::MAX cannot overflow or lose digits
/// through an intermediate floating-point conversion.
pub(crate) fn compact_count(count: u64) -> String {
    if count < 1000 {
        return count.to_string();
    }
    let scale = 10_u128.pow(count.ilog10().saturating_sub(3));
    let rounded = (u128::from(count) + scale / 2) / scale * scale;
    let (unit, suffix) = if rounded >= 1_000_000_000 {
        (1_000_000_000_u128, "B")
    } else if rounded >= 1_000_000 {
        (1_000_000_u128, "M")
    } else {
        (1000_u128, "K")
    };
    let whole = rounded / unit;
    let digits = 3_u32.saturating_sub(whole.ilog10()) as usize;
    if digits == 0 {
        format!("{whole} {suffix}")
    } else {
        let fraction = (rounded % unit) / (unit / 10_u128.pow(digits as u32));
        format!("{whole}.{fraction:0digits$} {suffix}")
    }
}

/// Dashboard elapsed times and sample averages, always in plain µs, ms or s.
/// Keep about three significant digits, including fractional microseconds;
/// choose the next unit if the displayed rounding would otherwise reach 1000.
pub(crate) fn sample_duration(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "—".into();
    }
    if seconds == 0.0 {
        return "0.00 µs".into();
    }
    let mut unit = if seconds < 0.001 {
        0
    } else if seconds < 1.0 {
        1
    } else {
        2
    };
    let decimals = |value: f64| {
        if value == 0.0 {
            2
        } else if value < 1.0 {
            (-value.log10()).ceil() as usize + 2
        } else if value < 10.0 {
            2
        } else if value < 100.0 {
            1
        } else {
            0
        }
    };
    loop {
        let (value, suffix) = match unit {
            0 => (seconds * 1e6, "µs"),
            1 => (seconds * 1e3, "ms"),
            _ => (seconds, "s"),
        };
        let mut digits = decimals(value);
        let text = format!("{value:.digits$}");
        let rounded: f64 = text.parse().expect("finite plain duration");
        if unit < 2 && rounded >= 1000.0 {
            unit += 1;
            continue;
        }
        // A carry into 1, 10 or 100 can also reduce the necessary decimals.
        digits = decimals(rounded);
        return format!("{value:.digits$} {suffix}");
    }
}

pub(crate) fn duration(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "—".into();
    }
    if seconds > 0.0 && seconds < 0.01 {
        let (mantissa, exponent) = parts(seconds, Some(2));
        return format!("{mantissa:.2}·10{} s", superscript(exponent));
    }
    let (value, unit) = if seconds >= 86_400.0 {
        (seconds / 86_400.0, "d")
    } else if seconds >= 3_600.0 {
        (seconds / 3_600.0, "h")
    } else if seconds >= 60.0 {
        (seconds / 60.0, "min")
    } else {
        (seconds, "s")
    };
    format!("{value:.2} {unit}")
}

pub(crate) fn relative_percent(ratio: f64) -> String {
    if !ratio.is_finite() || ratio < 0.0 {
        return "—".into();
    }
    if ratio <= 100.0 {
        return format!("{:.2}%", ratio * 100.0);
    }
    let (mantissa, exponent) = parts(ratio, Some(2));
    format!("{mantissa:.2}·10{}%", superscript(exponent + 2))
}

pub(crate) fn fraction(count: u64, total: u64) -> String {
    if total == 0 {
        "—".into()
    } else {
        format!("{:.1}%", 100.0 * count as f64 / total as f64)
    }
}
