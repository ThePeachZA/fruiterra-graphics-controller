//! Focus helpers for controls that are replaced when a view re-renders.

use dioxus::prelude::*;

/// Moves focus to an element by id once it exists in the DOM.
///
/// Inline confirmations and the player editor replace their trigger, and the
/// match dialog is unmounted rather than toggled, so the browser leaves focus
/// on `body`. The element may not be rendered yet when the state change is
/// handled, so the script retries on animation frames for about a second.
pub fn focus_element_later(id: &str) {
    let script = format!(
        "(() => {{
            let tries = 0;
            const tick = () => {{
                const element = document.getElementById('{id}');
                if (element) {{
                    element.focus();
                    return;
                }}
                if (tries++ < 60) requestAnimationFrame(tick);
            }};
            tick();
        }})();"
    );
    let _ = document::eval(&script);
}
