//! Application shell: providers, navigation, page chrome and status bar.

use dioxus::prelude::*;
use dioxus_icons::lucide::{
    Clock, Gamepad2, Images, Mic, MonitorPlay, PanelLeftClose, PanelLeftOpen, Pause, Play,
    Settings2, SlidersHorizontal, Square, Trophy, UsersRound,
};
use dioxus_primitives::toast::{ToastOptions, use_toast};
use fgc_core::{TimerAlignment, TimerMode};
use fgc_protocol::TimerActionDto;

use crate::components::input::Input;
use crate::components::switch::Switch;
use crate::components::toast::ToastProvider;
use crate::components::{Field, ToggleRow};
use crate::state::AppStore;
use crate::status::{status_text, status_tone};
use crate::theme::Destination;
use crate::timer::{
    TimerController, TimerState, broadcast_tick, format_clock, join_time, parse_non_negative,
    send_timer_action, split_time, target_countdown_script,
};
use crate::views::DestinationView;

const DX_THEME_CSS: Asset = asset!("/assets/dx-components-theme.css");
const THEME_CSS: Asset = asset!("/assets/theme.css");
const APP_CSS: Asset = asset!("/assets/app.css");
const COMPONENTS_CSS: Asset = asset!("/assets/components.css");
const OVERLAY_CSS: Asset = asset!("/assets/overlay.css");
const SETTINGS_CSS: Asset = asset!("/assets/settings.css");
const ASSETS_CSS: Asset = asset!("/assets/assets.css");
const TEAMS_CSS: Asset = asset!("/assets/teams.css");
const MATCH_CONTROLLER_CSS: Asset = asset!("/assets/match-controller.css");
const TOURNAMENT_CSS: Asset = asset!("/assets/tournament.css");
const PLACEHOLDER_CSS: Asset = asset!("/assets/placeholder.css");
// These vendored CSS modules are not injected into the desktop document on
// demand. Without explicit links, dialogs are unpositioned and inputs fall
// back to browser-default styling.
const DIALOG_CSS: Asset = asset!("/src/components/dialog/style.css");
const INPUT_CSS: Asset = asset!("/src/components/input/style.css");

const INTER_LATIN: Asset = asset!("/assets/fonts/Inter-latin.woff2");
const INTER_LATIN_EXT: Asset = asset!("/assets/fonts/Inter-latin-ext.woff2");

/// `@font-face` rules for the locally bundled Inter variable font.
///
/// Manganis does not rewrite `url()` references inside stylesheets, so the
/// rules are generated here from [`Asset`] constants and injected as a
/// `<style>` element.
const FONT_FACES: &str = r#"
@font-face {
    font-family: "Inter";
    font-style: normal;
    font-weight: 100 900;
    font-display: swap;
    src: url("__LATIN__") format("woff2");
    unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA,
        U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193,
        U+2212, U+2215, U+FEFF, U+FFFD;
}
@font-face {
    font-family: "Inter";
    font-style: normal;
    font-weight: 100 900;
    font-display: swap;
    src: url("__LATIN_EXT__") format("woff2");
    unicode-range: U+0100-02BA, U+02BD-02C5, U+02C7-02CC, U+02CE-02D7, U+02DD-02FF,
        U+0304, U+0308, U+0329, U+1D00-1DBF, U+1E00-1E9F, U+1EF2-1EFF, U+2020,
        U+20A0-20AB, U+20AD-20C0, U+2113, U+2C60-2C7F, U+A720-A7FF;
}
"#;

/// Root component.
#[component]
pub fn App() -> Element {
    let store = AppStore::use_store();
    use_context_provider(|| store);
    // The timer lives above the destination switch so it keeps counting while
    // the operator visits another destination.
    let timer = TimerController::use_controller();
    use_context_provider(|| timer);
    let mut destination = use_signal(|| Destination::Overlay);
    let mut compact_nav = use_signal(|| false);

    let font_faces = FONT_FACES
        .replace("__LATIN__", &INTER_LATIN.to_string())
        .replace("__LATIN_EXT__", &INTER_LATIN_EXT.to_string());

    rsx! {
        document::Link { rel: "stylesheet", href: DX_THEME_CSS }
        document::Link { rel: "stylesheet", href: THEME_CSS }
        document::Link { rel: "stylesheet", href: APP_CSS }
        document::Link { rel: "stylesheet", href: COMPONENTS_CSS }
        document::Link { rel: "stylesheet", href: OVERLAY_CSS }
        document::Link { rel: "stylesheet", href: SETTINGS_CSS }
        document::Link { rel: "stylesheet", href: ASSETS_CSS }
        document::Link { rel: "stylesheet", href: TEAMS_CSS }
        document::Link { rel: "stylesheet", href: MATCH_CONTROLLER_CSS }
        document::Link { rel: "stylesheet", href: TOURNAMENT_CSS }
        document::Link { rel: "stylesheet", href: PLACEHOLDER_CSS }
        document::Link { rel: "stylesheet", href: DIALOG_CSS }
        document::Link { rel: "stylesheet", href: INPUT_CSS }
        style { dangerous_inner_html: font_faces }
        ToastProvider {
            ErrorToasts {}
            NoticeToasts {}
            div { class: if compact_nav() { "fgc-app fgc-app--compact-nav" } else { "fgc-app" },
                Sidebar {
                    destination: destination(),
                    compact: compact_nav(),
                    on_navigate: move |next| destination.set(next),
                    on_toggle_compact: move |_| compact_nav.toggle(),
                }
                main { class: "fgc-main",
                    PageHeader { destination: destination() }
                    ErrorBanner {}
                    div { class: "fgc-content",
                        DestinationView { destination: destination() }
                    }
                    StatusBar {}
                }
                FloatingTimer {}
            }
        }
    }
}

/// Sidebar with the seven original destinations.
#[component]
fn Sidebar(
    destination: Destination,
    compact: bool,
    on_navigate: EventHandler<Destination>,
    on_toggle_compact: EventHandler<()>,
) -> Element {
    let toggle_label = if compact {
        "Expand navigation"
    } else {
        "Collapse navigation"
    };

    rsx! {
        aside { class: "fgc-sidebar",
            div { class: "fgc-brand",
                div { class: "fgc-brand__mark", "FG" }
                div { class: "fgc-brand__text",
                    p { class: "fgc-brand__name", "Fruiterra" }
                    p { class: "fgc-brand__sub", "Graphics Controller" }
                }
            }
            div { class: "fgc-sidebar__tools",
                button {
                    class: "fgc-nav-toggle",
                    r#type: "button",
                    aria_label: toggle_label,
                    title: toggle_label,
                    onclick: move |_| on_toggle_compact.call(()),
                    if compact {
                        PanelLeftOpen { size: 16 }
                    } else {
                        PanelLeftClose { size: 16 }
                    }
                    span { class: "fgc-nav-toggle__label", if compact { "Expand" } else { "Collapse" } }
                }
            }
            nav { class: "fgc-nav", aria_label: "Destinations",
                for item in Destination::ALL {
                    button {
                        key: "{item.label()}",
                        class: if item == destination {
                            "fgc-nav__item fgc-nav__item--active"
                        } else {
                            "fgc-nav__item"
                        },
                        r#type: "button",
                        aria_current: if item == destination { "page" } else { "false" },
                        aria_label: item.label(),
                        title: item.label(),
                        "data-label": item.label(),
                        onclick: move |_| on_navigate.call(item),
                        {nav_icon(item)}
                        span { class: "fgc-nav__label", "{item.label()}" }
                        if !item.is_implemented() {
                            span { class: "fgc-nav__soon", "{item.milestone()}" }
                        }
                    }
                }
            }
            div { class: "fgc-sidebar__footer",
                span { "Version {fgc_core::VERSION}" }
                span { "Overlays · 1920×1080" }
            }
        }
    }
}

/// Decorative Lucide icon for a navigation item. The button's text label
/// carries the accessible name, so the SVG has no accessible name of its own.
fn nav_icon(destination: Destination) -> Element {
    let class = "fgc-nav__icon";
    let size = 18;
    match destination {
        Destination::MatchController => rsx! { Gamepad2 { class, size } },
        Destination::Overlay => rsx! { MonitorPlay { class, size } },
        Destination::Teams => rsx! { UsersRound { class, size } },
        Destination::Talent => rsx! { Mic { class, size } },
        Destination::Assets => rsx! { Images { class, size } },
        Destination::Tournament => rsx! { Trophy { class, size } },
        Destination::Settings => rsx! { Settings2 { class, size } },
    }
}

/// Page title, blurb and milestone pill for the current destination.
#[component]
fn PageHeader(destination: Destination) -> Element {
    let pill = if destination.is_implemented() {
        "Live"
    } else {
        destination.milestone()
    };

    rsx! {
        header { class: "fgc-page-header",
            div {
                p { class: "fgc-eyebrow", "Fruiterra Graphics Controller" }
                h1 { class: "fgc-page-header__title", "{destination.label()}" }
                p { class: "fgc-page-header__blurb", "{destination.blurb()}" }
            }
            div { class: "fgc-page-header__actions",
                span { class: "fgc-pill", "{pill}" }
            }
        }
    }
}

/// Dismissible banner for the most recent user-facing error.
#[component]
fn ErrorBanner() -> Element {
    let store = use_context::<AppStore>();
    let Some(message) = store.last_error() else {
        return rsx! {};
    };

    rsx! {
        div { class: "fgc-banner", role: "alert",
            div { class: "fgc-banner__body",
                p { class: "fgc-banner__title", "Something needs attention" }
                p { class: "fgc-banner__message", "{message}" }
            }
            button {
                class: "fgc-banner__close",
                r#type: "button",
                aria_label: "Dismiss error",
                onclick: move |_| store.dismiss_error(),
                "×"
            }
        }
    }
}

/// Surfaces new user-facing errors as toasts.
#[component]
fn ErrorToasts() -> Element {
    let store = use_context::<AppStore>();
    let toast = use_toast();
    let mut seen = use_signal(|| None::<String>);

    use_effect(move || {
        let current = store.last_error();
        if current != *seen.peek() {
            if let Some(message) = &current {
                toast.error(
                    "Something went wrong".to_owned(),
                    ToastOptions::new().description(message.clone()),
                );
            }
            seen.set(current);
        }
    });

    rsx! {}
}

/// Surfaces new success messages as toasts.
#[component]
fn NoticeToasts() -> Element {
    let store = use_context::<AppStore>();
    let toast = use_toast();
    let mut seen = use_signal(|| None::<String>);

    use_effect(move || {
        let current = store.notice();
        if current != *seen.peek() {
            if let Some(message) = &current {
                toast.success(message.clone(), ToastOptions::new());
                store.dismiss_notice();
            }
            seen.set(current);
        }
    });

    rsx! {}
}

/// Compact always-on timer control pinned above the status bar.
///
/// The icon row starts, pauses and stops the clock from any destination.
/// Expanding the sliders button reveals the settings that used to live in
/// Settings and the Match Controller Timer tab. All settings still go through
/// [`AppStore`], and the run state stays in [`TimerController`], so the widget
/// and the overlay always agree.
#[component]
fn FloatingTimer() -> Element {
    let store = use_context::<AppStore>();
    let mut timer = use_context::<TimerController>();
    let settings = store.state().read().settings.timer.clone();
    let scheduled_start = store
        .state()
        .read()
        .current_match
        .as_ref()
        .and_then(|current| current.scheduled_at.clone());
    let current = (timer.state)();
    let target = (timer.target)();
    let duration = (timer.duration)();
    let mut expanded = use_signal(|| false);
    let mode = settings.mode;
    let target_value = if mode == TimerMode::Scheduled {
        scheduled_start.clone().unwrap_or_default()
    } else {
        target.clone()
    };

    // Keep the shared target signal in step with the current match's scheduled
    // start. The tick loop then reuses the same countdown machinery as the
    // manual target mode without persisting a second copy of the date. Read the
    // mode from the store here so switching modes reactively reruns the effect.
    use_effect(move || {
        if store.state().read().settings.timer.mode != TimerMode::Scheduled {
            return;
        }
        let Some(scheduled) = store
            .state()
            .read()
            .current_match
            .as_ref()
            .and_then(|current| current.scheduled_at.clone())
        else {
            (timer.target).set(String::new());
            return;
        };
        (timer.target).set(scheduled.clone());
        let mut timer_state = timer.state;
        spawn(async move {
            match document::eval(&target_countdown_script(&scheduled))
                .join::<i64>()
                .await
            {
                Ok(seconds) => timer_state.set(TimerState {
                    running: false,
                    total_seconds: u32::try_from(seconds).unwrap_or(0),
                }),
                Err(err) => tracing::warn!(%err, %scheduled, "scheduled timer eval failed"),
            }
        });
    });

    let countdown_fields = if mode == TimerMode::CountDown {
        let (hours, minutes, seconds) = split_time(duration);
        rsx! {
            div { class: "fgc-float-timer__duration",
                label { class: "fgc-field",
                    span { class: "fgc-field__label", "Hours" }
                    Input {
                        r#type: "number",
                        min: "0",
                        value: "{hours}",
                        aria_label: "Countdown hours",
                        oninput: move |event: FormEvent| {
                            let (_, minutes, seconds) = split_time((timer.duration)());
                            set_duration(store, timer, join_time(parse_non_negative(&event.value()), minutes, seconds));
                        },
                    }
                }
                label { class: "fgc-field",
                    span { class: "fgc-field__label", "Minutes" }
                    Input {
                        r#type: "number",
                        min: "0",
                        value: "{minutes}",
                        aria_label: "Countdown minutes",
                        oninput: move |event: FormEvent| {
                            let (hours, _, seconds) = split_time((timer.duration)());
                            set_duration(store, timer, join_time(hours, parse_non_negative(&event.value()), seconds));
                        },
                    }
                }
                label { class: "fgc-field",
                    span { class: "fgc-field__label", "Seconds" }
                    Input {
                        r#type: "number",
                        min: "0",
                        value: "{seconds}",
                        aria_label: "Countdown seconds",
                        oninput: move |event: FormEvent| {
                            let (hours, minutes, _) = split_time((timer.duration)());
                            set_duration(store, timer, join_time(hours, minutes, parse_non_negative(&event.value())));
                        },
                    }
                }
            }
        }
    } else {
        rsx! {}
    };

    rsx! {
        div { class: "fgc-float-timer", role: "group", aria_label: "Timer control",
            div { class: "fgc-float-timer__clock",
                Clock { class: "fgc-float-timer__icon", size: 16 }
                span {
                    class: if current.running {
                        "fgc-float-timer__value fgc-float-timer__value--running"
                    } else {
                        "fgc-float-timer__value"
                    },
                    "{format_clock(current.total_seconds)}"
                }
            }
            div { class: "fgc-float-timer__actions",
                button {
                    r#type: "button",
                    class: "fgc-float-timer__icon-button fgc-float-timer__icon-button--start",
                    aria_label: "Start timer",
                    title: "Start",
                    disabled: current.running
                        || (matches!(mode, TimerMode::Target | TimerMode::Scheduled)
                            && target_value.trim().is_empty()),
                    onclick: move |_| start_timer(store, timer),
                    Play { size: 15 }
                }
                button {
                    r#type: "button",
                    class: "fgc-float-timer__icon-button",
                    aria_label: "Pause timer",
                    title: "Pause",
                    disabled: !current.running,
                    onclick: move |_| pause_timer(store, timer),
                    Pause { size: 15 }
                }
                button {
                    r#type: "button",
                    class: "fgc-float-timer__icon-button fgc-float-timer__icon-button--stop",
                    aria_label: "Stop timer",
                    title: "Stop",
                    onclick: move |_| stop_timer(store, timer),
                    Square { size: 13 }
                }
                button {
                    r#type: "button",
                    class: "fgc-float-timer__icon-button fgc-float-timer__icon-button--settings",
                    aria_label: "Timer settings",
                    aria_expanded: expanded(),
                    aria_controls: "fgc-float-timer-settings",
                    title: "Timer settings",
                    onclick: move |_| {
                        let next = !expanded();
                        expanded.set(next);
                    },
                    SlidersHorizontal { size: 15 }
                }
            }
            if expanded() {
                div { id: "fgc-float-timer-settings", class: "fgc-float-timer__popup",
                    div { class: "fgc-float-timer__popup-header",
                        span { "Timer settings" }
                        button {
                            r#type: "button",
                            class: "fgc-float-timer__close",
                            aria_label: "Close timer settings",
                            onclick: move |_| expanded.set(false),
                            "×"
                        }
                    }
                    Field { label: "Mode".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Timer mode",
                            value: match mode {
                                TimerMode::CountUp => "countup",
                                TimerMode::CountDown => "countdown",
                                TimerMode::Target => "target",
                                TimerMode::Scheduled => "scheduled",
                            },
                            onchange: move |event| {
                                let next = match event.value().as_str() {
                                    "countdown" => TimerMode::CountDown,
                                    "target" => TimerMode::Target,
                                    "scheduled" => TimerMode::Scheduled,
                                    _ => TimerMode::CountUp,
                                };
                                store.set_timer_mode(next);
                                set_timer_seconds(store, timer, initial_seconds(timer, next));
                            },
                            option { value: "countup", "Count up" }
                            option { value: "countdown", "Count down" }
                            option { value: "target", "Count to target" }
                            option { value: "scheduled", "Countdown to scheduled match" }
                        }
                    }
                    {countdown_fields}
                    if mode == TimerMode::Target {
                        Field {
                            label: "Target date and time".to_owned(),
                            hint: Some("Set a target before starting.".to_owned()),
                            Input {
                                r#type: "datetime-local",
                                value: "{target}",
                                aria_label: "Target date and time",
                                oninput: move |event: FormEvent| {
                                    (timer.target).set(event.value());
                                },
                            }
                        }
                    }
                    if mode == TimerMode::Scheduled {
                        Field {
                            label: "Scheduled start".to_owned(),
                            hint: Some("Set this on the Match tab.".to_owned()),
                            p { class: "fgc-field__value",
                                if let Some(scheduled) = scheduled_start.as_deref() {
                                    "{scheduled}"
                                } else {
                                    "Not scheduled"
                                }
                            }
                        }
                    }
                    Field { label: "Alignment".to_owned(),
                        select {
                            class: "fgc-native-select",
                            aria_label: "Timer alignment",
                            value: match settings.alignment {
                                TimerAlignment::Left => "left",
                                TimerAlignment::Center => "center",
                                TimerAlignment::Right => "right",
                            },
                            onchange: move |event| {
                                let alignment = match event.value().as_str() {
                                    "left" => TimerAlignment::Left,
                                    "right" => TimerAlignment::Right,
                                    _ => TimerAlignment::Center,
                                };
                                store.set_timer_alignment(alignment);
                            },
                            option { value: "left", "Left" }
                            option { value: "center", "Centre" }
                            option { value: "right", "Right" }
                        }
                    }
                    ToggleRow {
                        title: "Show hours".to_owned(),
                        hint: Some("When off, hours are added to the minutes.".to_owned()),
                        Switch {
                            checked: settings.show_hours,
                            aria_label: "Show hours",
                            on_checked_change: move |checked| {
                                store.set_timer_show_hours(checked);
                                let settings = store.state().peek().settings.timer.clone();
                                broadcast_tick(&store, &settings, timer.state.peek().total_seconds);
                            },
                        }
                    }
                    ToggleRow {
                        title: "Leading zeroes".to_owned(),
                        hint: Some("Pad single-digit values on the overlay.".to_owned()),
                        Switch {
                            checked: settings.leading_zeroes,
                            aria_label: "Leading zeroes",
                            on_checked_change: move |checked| store.set_timer_leading_zeroes(checked),
                        }
                    }
                    ToggleRow {
                        title: "Standalone timer".to_owned(),
                        hint: Some("Hide the timer surround.".to_owned()),
                        Switch {
                            checked: settings.standalone,
                            aria_label: "Standalone timer",
                            on_checked_change: move |checked| store.set_timer_standalone(checked),
                        }
                    }
                }
            }
        }
    }
}

/// Initial seconds shown when the counting mode changes.
fn initial_seconds(timer: TimerController, mode: TimerMode) -> u32 {
    match mode {
        TimerMode::CountDown => (timer.duration)(),
        TimerMode::CountUp | TimerMode::Target | TimerMode::Scheduled => 0,
    }
}

/// Starts the clock from the current display value, broadcasting the initial
/// count-down value first so the overlay cannot show stale time.
fn start_timer(store: AppStore, mut timer: TimerController) {
    let mode = store.state().peek().settings.timer.mode;
    let mut seconds = (timer.state).peek().total_seconds;
    if mode == TimerMode::CountDown && seconds == 0 {
        seconds = (timer.duration)();
    }
    if mode == TimerMode::CountUp && (timer.state).peek().total_seconds == 0 {
        // Count-up starts from zero by definition; no reset frame is needed.
    } else if mode != TimerMode::CountUp {
        let (hours, minutes, secs) = split_time(seconds);
        send_timer_action(&store, TimerActionDto::reset(hours, minutes, secs));
    }
    (timer.state).set(TimerState {
        running: true,
        total_seconds: seconds,
    });
    send_timer_action(&store, TimerActionDto::start());
}

/// Pauses the clock without changing the value shown.
fn pause_timer(store: AppStore, mut timer: TimerController) {
    let total_seconds = (timer.state).peek().total_seconds;
    (timer.state).set(TimerState {
        running: false,
        total_seconds,
    });
    send_timer_action(&store, TimerActionDto::pause());
}

/// Stops the clock and returns it to the configured starting value.
fn stop_timer(store: AppStore, timer: TimerController) {
    set_timer_seconds(
        store,
        timer,
        initial_seconds(timer, store.state().peek().settings.timer.mode),
    );
}

/// Sets the displayed seconds and tells the overlay to show the same value.
fn set_timer_seconds(store: AppStore, mut timer: TimerController, total_seconds: u32) {
    let (hours, minutes, seconds) = split_time(total_seconds);
    (timer.state).set(TimerState {
        running: false,
        total_seconds,
    });
    send_timer_action(&store, TimerActionDto::reset(hours, minutes, seconds));
}

/// Updates the count-down duration and previews it in the widget.
fn set_duration(store: AppStore, mut timer: TimerController, total_seconds: u32) {
    (timer.duration).set(total_seconds);
    set_timer_seconds(store, timer, total_seconds);
}

/// Bottom status bar with server, asset and save state.
#[component]
fn StatusBar() -> Element {
    let store = use_context::<AppStore>();
    let server = store.server();
    let status = server.status();
    let tone = status_tone(&status);
    let label = status_text(&status);
    let base_url = if store.urls_visible() {
        server.base_url()
    } else {
        "URL hidden".to_owned()
    };
    let assets = store.asset_count();
    let dirty = store.is_dirty();
    let saving = store.is_saving();

    rsx! {
        footer { class: "fgc-statusbar",
            span { class: "fgc-statusbar__group",
                span { class: "fgc-dot fgc-dot--{tone}" }
                span { "{label}" }
            }
            span { class: "fgc-statusbar__group fgc-mono", "{base_url}" }
            span { class: "fgc-statusbar__spacer" }
            span { class: "fgc-statusbar__group", "{assets} assets" }
            span { class: "fgc-statusbar__group",
                if saving {
                    "Saving…"
                } else if dirty {
                    "Unsaved changes"
                } else {
                    "Saved"
                }
            }
        }
    }
}
