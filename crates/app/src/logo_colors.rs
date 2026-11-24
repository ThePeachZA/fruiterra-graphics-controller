//! Dominant-colour sampling for imported team logos.
//!
//! The image bytes are already in the browser as a data URL by the time the
//! operator picks a logo, so sampling happens on a small canvas instead of
//! adding an image decoder to the desktop bundle. A data URL does not taint
//! the canvas, and the 32×32 downscale keeps the sample cheap. The sampled
//! palette yields a recommended background and text colour, and the Rust side
//! adjusts the text towards the logo's own hue when it must stay legible on a
//! manually chosen background.

use dioxus::prelude::*;

type Rgb = [u8; 3];

/// The recommended colours and sampled palette for one logo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogoPalette {
    background: String,
    samples: Vec<Rgb>,
}

impl LogoPalette {
    /// The recommended background colour for this logo.
    #[must_use]
    pub fn background(&self) -> &str {
        &self.background
    }

    /// Picks a logo-harmonised text colour for the supplied background.
    ///
    /// `strict` requests the 4.5:1 contrast target used by computed contrast;
    /// otherwise the 3:1 target for large display text is used.
    #[must_use]
    pub fn text_for_background(&self, background: &str, strict: bool) -> Option<String> {
        let background = parse_hex_rgb(background)?;
        let text = harmonize_candidate(&self.samples, background, strict);
        Some(hex_color(&text))
    }
}

/// Samples the dominant palette of `data_url` and returns recommended colours.
pub async fn sample_logo(data_url: &str) -> Option<LogoPalette> {
    let source = serde_json::to_string(data_url).ok()?;
    let script = format!(
        r#"
return await (async () => {{
    const img = new Image();
    img.src = {source};
    try {{ await img.decode(); }} catch (_) {{ return null; }}
    const canvas = document.createElement('canvas');
    canvas.width = 32;
    canvas.height = 32;
    const ctx = canvas.getContext('2d');
    if (!ctx) return null;
    ctx.drawImage(img, 0, 0, 32, 32);
    let pixels;
    try {{ pixels = ctx.getImageData(0, 0, 32, 32).data; }} catch (_) {{ return null; }}
    const buckets = new Map();
    for (let i = 0; i < pixels.length; i += 4) {{
        if (pixels[i + 3] < 128) continue;
        const r = pixels[i], g = pixels[i + 1], b = pixels[i + 2];
        const key = ((r >> 4) << 8) | ((g >> 4) << 4) | (b >> 4);
        const bucket = buckets.get(key) || {{ count: 0, r: 0, g: 0, b: 0 }};
        bucket.count += 1;
        bucket.r += r;
        bucket.g += g;
        bucket.b += b;
        buckets.set(key, bucket);
    }}
    const ranked = Array.from(buckets.values())
        .sort((a, b) => b.count - a.count)
        .slice(0, 8)
        .map((bucket) => {{
            const hex = (value) => Math.round(value / bucket.count)
                .toString(16)
                .padStart(2, '0');
            return '#' + hex(bucket.r) + hex(bucket.g) + hex(bucket.b);
        }});
    return ranked.length ? ranked.join('|') : null;
}})()
"#
    );

    let value = match document::eval(&script).join::<Option<String>>().await {
        Ok(Some(value)) => value,
        Ok(None) => return None,
        Err(error) => {
            tracing::warn!(%error, "logo colour sampling failed");
            return None;
        }
    };
    let samples = parse_samples(&value)?;
    let background = hex_color(&choose_background(&samples));
    let palette = LogoPalette {
        background,
        samples,
    };
    let _recommended_text = palette.text_for_background(palette.background(), true)?;
    Some(palette)
}

/// Derives the Night Broadcast contrast colour for a six-digit hex background.
///
/// The threshold matches the roster overlays, so manual and computed teams
/// render the same text colour whether the value is stored or recomputed.
#[must_use]
pub fn contrast_text_color(background: &str) -> Option<String> {
    let background = parse_hex_rgb(background)?;
    let luminance = weighted_luminance(background);
    Some(
        if luminance > 0.5 {
            "#0b0d12"
        } else {
            "#eef2f8"
        }
        .to_owned(),
    )
}

/// Keeps an existing logo-derived text colour legible on a new background.
///
/// The hue is retained where possible; the colour only moves towards black or
/// white far enough to satisfy the 4.5:1 or 3:1 display-text target.
#[must_use]
pub fn harmonize_text_color(text: &str, background: &str, strict: bool) -> Option<String> {
    let text = parse_hex_rgb(text)?;
    let background = parse_hex_rgb(background)?;
    let target = contrast_target(strict);
    Some(hex_color(&adjust_for_contrast(text, background, target)))
}

fn parse_samples(value: &str) -> Option<Vec<Rgb>> {
    let samples = value
        .split('|')
        .map(parse_hex_rgb)
        .collect::<Option<Vec<_>>>()?;
    (!samples.is_empty()).then_some(samples)
}

fn parse_hex_rgb(value: &str) -> Option<Rgb> {
    if value.len() != 7 || !value.starts_with('#') {
        return None;
    }
    let digits = &value.as_bytes()[1..];
    if !digits.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let channel = |start: usize| u8::from_str_radix(&value[1 + start..3 + start], 16).ok();
    Some([channel(0)?, channel(2)?, channel(4)?])
}

fn hex_color(rgb: &Rgb) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

fn choose_background(samples: &[Rgb]) -> Rgb {
    let mut best = samples[0];
    let mut best_score = f64::NEG_INFINITY;
    for &sample in samples {
        let saturation = saturation(sample);
        let luminance = relative_luminance(sample);
        let score = saturation * 2.0 + 1.0 - (luminance - 0.45).abs() * 1.6;
        if score > best_score {
            best = sample;
            best_score = score;
        }
    }
    best
}

fn harmonize_candidate(samples: &[Rgb], background: Rgb, strict: bool) -> Rgb {
    let mut best: Option<Rgb> = None;
    let mut best_score = f64::NEG_INFINITY;
    for &sample in samples {
        let contrast = contrast_ratio(sample, background);
        let distance = color_distance(sample, background);
        let score = contrast * 2.0 + distance / 64.0;
        if contrast > 1.15 && score > best_score {
            best = Some(sample);
            best_score = score;
        }
    }
    let base = best.unwrap_or_else(|| {
        let vivid = samples
            .iter()
            .copied()
            .max_by(|a, b| saturation(*a).total_cmp(&saturation(*b)))
            .unwrap_or(background);
        if relative_luminance(background) > 0.45 {
            mix(vivid, [0, 0, 0], 0.75)
        } else {
            mix(vivid, [255, 255, 255], 0.75)
        }
    });
    adjust_for_contrast(base, background, contrast_target(strict))
}

fn adjust_for_contrast(text: Rgb, background: Rgb, target: f64) -> Rgb {
    if contrast_ratio(text, background) >= target {
        return text;
    }
    let lighten = relative_luminance(background) <= 0.45;
    let meets = |scale: f64| contrast_ratio(scale_rgb(text, scale), background) >= target;
    let candidate = if lighten {
        let brightest = text.into_iter().max().unwrap_or(0);
        if brightest == 0 {
            return [238, 242, 248];
        }
        let mut low = 1.0;
        let mut high = 255.0 / f64::from(brightest);
        for _ in 0..32 {
            let middle = (low + high) / 2.0;
            if meets(middle) {
                high = middle;
            } else {
                low = middle;
            }
        }
        scale_rgb(text, high)
    } else {
        let mut low = 0.0;
        let mut high = 1.0;
        if !meets(low) {
            return [11, 13, 18];
        }
        for _ in 0..32 {
            let middle = (low + high) / 2.0;
            if meets(middle) {
                low = middle;
            } else {
                high = middle;
            }
        }
        scale_rgb(text, low)
    };
    if contrast_ratio(candidate, background) >= target {
        candidate
    } else if lighten {
        [238, 242, 248]
    } else {
        [11, 13, 18]
    }
}

fn contrast_target(strict: bool) -> f64 {
    if strict { 4.5 } else { 3.0 }
}

fn mix(from: Rgb, to: Rgb, amount: f64) -> Rgb {
    let channel = |index: usize| {
        let value = f64::from(from[index]) * (1.0 - amount) + f64::from(to[index]) * amount;
        value.round().clamp(0.0, 255.0) as u8
    };
    [channel(0), channel(1), channel(2)]
}

fn scale_rgb(rgb: Rgb, scale: f64) -> Rgb {
    let channel = |value: u8| (f64::from(value) * scale).round().clamp(0.0, 255.0) as u8;
    [channel(rgb[0]), channel(rgb[1]), channel(rgb[2])]
}

fn color_distance(a: Rgb, b: Rgb) -> f64 {
    let square = |index: usize| {
        let delta = f64::from(a[index]) - f64::from(b[index]);
        delta * delta
    };
    (square(0) + square(1) + square(2)).sqrt()
}

fn saturation(rgb: Rgb) -> f64 {
    let high = rgb.into_iter().max().unwrap_or(0);
    let low = rgb.into_iter().min().unwrap_or(0);
    if high == 0 {
        0.0
    } else {
        f64::from(high - low) / f64::from(high)
    }
}

fn relative_luminance(rgb: Rgb) -> f64 {
    let channel = |value: u8| {
        let value = f64::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(rgb[0]) + 0.7152 * channel(rgb[1]) + 0.0722 * channel(rgb[2])
}

fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let a = relative_luminance(a);
    let b = relative_luminance(b);
    let lighter = a.max(b);
    let darker = a.min(b);
    (lighter + 0.05) / (darker + 0.05)
}

fn weighted_luminance(rgb: Rgb) -> f64 {
    (0.299 * f64::from(rgb[0]) + 0.587 * f64::from(rgb[1]) + 0.114 * f64::from(rgb[2])) / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_six_digit_hex_colours() {
        assert!(parse_hex_rgb("#FF7B1A").is_some());
        assert!(parse_hex_rgb("#000000").is_some());
        assert!(parse_hex_rgb("#FFF").is_none());
        assert!(parse_hex_rgb("FF0000").is_none());
        assert!(parse_hex_rgb("#GG0000").is_none());
    }

    #[test]
    fn parses_a_sampled_palette() {
        let samples = parse_samples("#FF7B1A|#1B4DFF|#F4F4F4").expect("palette");
        assert_eq!(samples.len(), 3);
        assert_eq!(samples[0], [0xFF, 0x7B, 0x1A]);
        assert!(parse_samples("").is_none());
        assert!(parse_samples("#FFF|#000000").is_none());
    }

    #[test]
    fn recommends_a_saturated_logo_colour_for_the_background() {
        let samples = parse_samples("#F4F4F4|#032248|#FFFFFF|#032248").expect("palette");
        assert_eq!(hex_color(&choose_background(&samples)), "#032248");
    }

    #[test]
    fn harmonizes_a_similar_text_colour_without_losing_the_logo_hue() {
        let text = harmonize_text_color("#032248", "#032248", false).expect("text");
        let text_rgb = parse_hex_rgb(&text).expect("hex");
        let background = parse_hex_rgb("#032248").expect("hex");
        assert!(contrast_ratio(text_rgb, background) >= 3.0);
        assert!(text_rgb[2] > background[2]);
        assert_eq!(text_rgb[2], *text_rgb.iter().max().expect("channel"));
        assert!(text_rgb[0] * 4 < text_rgb[2]);
    }

    #[test]
    fn keeps_a_monochrome_logo_hue_when_computed_contrast_is_required() {
        let palette = LogoPalette {
            background: "#8309C4".to_owned(),
            samples: parse_samples("#9265A4|#8309C4|#8C41B0").expect("palette"),
        };
        let text = palette
            .text_for_background("#8309C4", true)
            .expect("harmonised text");
        let text_rgb = parse_hex_rgb(&text).expect("hex");
        let background = parse_hex_rgb("#8309C4").expect("hex");
        assert!(contrast_ratio(text_rgb, background) >= 4.5);
        assert!(text_rgb[2] > text_rgb[1]);
        assert!(text_rgb[0] > background[0]);
    }

    #[test]
    fn uses_a_higher_target_for_computed_contrast() {
        let text = harmonize_text_color("#8C41AF", "#8C41AF", true).expect("text");
        let text_rgb = parse_hex_rgb(&text).expect("hex");
        let background = parse_hex_rgb("#8C41AF").expect("hex");
        assert!(contrast_ratio(text_rgb, background) >= 4.5);
    }

    #[test]
    fn derives_the_same_contrast_colours_as_the_rosters() {
        assert_eq!(contrast_text_color("#FFFFFF").as_deref(), Some("#0b0d12"));
        assert_eq!(contrast_text_color("#000000").as_deref(), Some("#eef2f8"));
        assert_eq!(contrast_text_color("#ff5c7a").as_deref(), Some("#0b0d12"));
        assert_eq!(contrast_text_color("#3ecf8e").as_deref(), Some("#0b0d12"));
        assert!(contrast_text_color("#FFF").is_none());
        assert!(contrast_text_color("red").is_none());
    }
}
