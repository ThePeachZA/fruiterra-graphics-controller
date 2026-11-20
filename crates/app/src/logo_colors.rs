//! Dominant-colour sampling for imported team logos.
//!
//! The image bytes are already in the browser as a data URL by the time the
//! operator picks a logo, so sampling happens on a small canvas instead of
//! adding an image decoder to the desktop bundle. A data URL does not taint
//! the canvas, and the 32×32 downscale keeps the sample cheap.

use dioxus::prelude::*;

/// Samples the dominant colour of `data_url` and returns a primary colour plus
/// a contrasting text colour. Returns `None` for transparent or neutral-only
/// images, which are common for monochrome marks.
pub async fn sample_logo(data_url: &str) -> Option<(String, String)> {
    let source = serde_json::to_string(data_url).ok()?;
    let script = format!(
        r#"
(async () => {{
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
        const hi = Math.max(r, g, b), lo = Math.min(r, g, b);
        if (hi > 245 || lo < 10 || hi - lo < 12) continue;
        const key = ((r >> 4) << 8) | ((g >> 4) << 4) | (b >> 4);
        const bucket = buckets.get(key) || {{ count: 0, r: 0, g: 0, b: 0 }};
        bucket.count += 1;
        bucket.r += r;
        bucket.g += g;
        bucket.b += b;
        buckets.set(key, bucket);
    }}
    let best = null;
    for (const bucket of buckets.values()) {{
        if (!best || bucket.count > best.count) best = bucket;
    }}
    if (!best) return null;
    const r = Math.round(best.r / best.count);
    const g = Math.round(best.g / best.count);
    const b = Math.round(best.b / best.count);
    const hex = '#' + [r, g, b]
        .map((value) => value.toString(16).padStart(2, '0'))
        .join('')
        .toUpperCase();
    const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
    const text = luminance > 0.55 ? '#000000' : '#FFFFFF';
    return hex + '|' + text;
}})()
"#
    );

    let SampledColors { hex, text } = match document::eval(&script).join::<Option<String>>().await {
        Ok(Some(value)) => parse_sample(&value)?,
        Ok(None) => return None,
        Err(error) => {
            tracing::warn!(%error, "logo colour sampling failed");
            return None;
        }
    };

    Some((hex, text))
}

struct SampledColors {
    hex: String,
    text: String,
}

fn parse_sample(value: &str) -> Option<SampledColors> {
    let (hex, text) = value.split_once('|')?;
    if !is_hex_color(hex) || (text != "#000000" && text != "#FFFFFF") {
        return None;
    }
    Some(SampledColors {
        hex: hex.to_owned(),
        text: text.to_owned(),
    })
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_six_digit_hex_colours() {
        assert!(is_hex_color("#FF7B1A"));
        assert!(is_hex_color("#000000"));
        assert!(!is_hex_color("#FFF"));
        assert!(!is_hex_color("FF0000"));
        assert!(!is_hex_color("#GG0000"));
    }

    #[test]
    fn parses_a_sampled_colour_pair() {
        let colors = parse_sample("#FF7B1A|#000000").expect("sample");
        assert_eq!(colors.hex, "#FF7B1A");
        assert_eq!(colors.text, "#000000");
        assert!(parse_sample("#FFF|#000000").is_none());
        assert!(parse_sample("#FF7B1A|#123456").is_none());
        assert!(parse_sample("#FF7B1A").is_none());
    }
}
