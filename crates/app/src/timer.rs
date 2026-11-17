//! Ephemeral timer state and the one-second tick loop.
//!
//! The clock has to keep counting while the operator visits another
//! destination, so its state and the loop that advances it live above the
//! destination switch and are shared through context. Only the timer
//! *settings* are persisted; run state is never saved.

use std::time::Duration;

use dioxus::prelude::*;
use fgc_core::{TimerMode, TimerSettings};
use fgc_protocol::{TimerActionDto, timer_action, timer_tick};
use fgc_server::Outbound;

use crate::state::AppStore;

/// Ephemeral timer run state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimerState {
    /// Whether the clock is advancing.
    pub running: bool,
    /// Whole seconds currently shown.
    pub total_seconds: u32,
}

/// Timer signals shared by the root, the tick loop and the Timer tab.
#[derive(Clone, Copy)]
pub struct TimerController {
    /// Run state and the value shown on the Timer tab.
    pub state: Signal<TimerState>,
    /// Count-to-target value as typed into the `datetime-local` field.
    pub target: Signal<String>,
}

impl TimerController {
    /// Creates the signals and starts the tick loop.
    ///
    /// Call this once from the root component. Keeping the loop there means
    /// navigating to another destination neither stops the broadcast nor
    /// resets the clock.
    pub fn use_controller() -> Self {
        let store = use_context::<AppStore>();
        let app_state = store.state();
        let controller = Self {
            state: use_signal(TimerState::default),
            target: use_signal(String::new),
        };

        use_future(move || async move {
            let mut timer = controller.state;
            let target = controller.target;
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                if !timer.peek().running {
                    continue;
                }

                let (mode, settings) = {
                    let state = app_state.peek();
                    (state.settings.timer.mode, state.settings.timer.clone())
                };
                let current = *timer.peek();
                let next = match mode {
                    TimerMode::CountUp => current.total_seconds.saturating_add(1),
                    TimerMode::CountDown => current.total_seconds.saturating_sub(1),
                    TimerMode::Target => {
                        let target = target.peek().clone();
                        if target.is_empty() {
                            0
                        } else {
                            match document::eval(&target_countdown_script(&target))
                                .join::<i64>()
                                .await
                            {
                                Ok(seconds) => u32::try_from(seconds).unwrap_or(0),
                                Err(err) => {
                                    tracing::warn!(target = %target, %err, "timer target eval failed");
                                    0
                                }
                            }
                        }
                    }
                };

                if matches!(mode, TimerMode::CountDown | TimerMode::Target) && next == 0 {
                    timer.set(TimerState {
                        running: false,
                        total_seconds: 0,
                    });
                    broadcast_tick(&store, &settings, 0);
                    continue;
                }

                timer.set(TimerState {
                    running: true,
                    total_seconds: next,
                });
                broadcast_tick(&store, &settings, next);
            }
        });

        controller
    }
}

/// Splits a total into hours, minutes and seconds.
pub fn split_time(total: u32) -> (u32, u32, u32) {
    (total / 3600, (total % 3600) / 60, total % 60)
}

/// Combines hours, minutes and seconds into a total.
pub fn join_time(hours: u32, minutes: u32, seconds: u32) -> u32 {
    hours
        .saturating_mul(3600)
        .saturating_add(minutes.saturating_mul(60))
        .saturating_add(seconds)
}

/// Formats the preview clock as `HH:MM:SS`.
pub fn format_clock(total: u32) -> String {
    let (hours, minutes, seconds) = split_time(total);
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

/// Builds the JavaScript that returns the whole seconds remaining until `target`.
///
/// Dioxus evaluates a script as an async function body, so the value must be
/// returned explicitly: a trailing expression would yield `undefined` and the
/// eval result would fail to deserialize.
pub fn target_countdown_script(target: &str) -> String {
    format!("return Math.max(0, Math.floor((Date.parse({target:?}) - Date.now()) / 1000));")
}

/// Broadcasts one timer tick when the server is running.
pub fn broadcast_tick(store: &AppStore, settings: &TimerSettings, total: u32) {
    if !store.server().is_running() {
        return;
    }
    let (hours, minutes, seconds) = split_time(total);
    let message = timer_tick(settings, hours, minutes, seconds);
    if let Ok(outbound) = Outbound::from_message(&message) {
        store.broadcast_timer(outbound);
    }
}

/// Broadcasts one timer control action when the server is running.
pub fn send_timer_action(store: &AppStore, action: TimerActionDto) {
    if !store.server().is_running() {
        return;
    }
    let message = timer_action(action);
    if let Ok(outbound) = Outbound::from_message(&message) {
        store.broadcast_timer(outbound);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_helpers_round_trip() {
        assert_eq!(split_time(3661), (1, 1, 1));
        assert_eq!(join_time(1, 1, 1), 3661);
        assert_eq!(format_clock(3661), "01:01:01");
    }

    #[test]
    fn target_script_returns_a_number() {
        // The desktop eval wrapper runs the script as an async function body, so
        // the result must be returned; a trailing expression evaluates to
        // `undefined` and the timer silently stops.
        let script = target_countdown_script("2026-11-15T18:12");
        assert!(script.starts_with("return "), "{script}");
        assert!(script.contains("\"2026-11-15T18:12\""), "{script}");
        assert!(script.contains("Date.parse"), "{script}");
    }
}
