//! OBS-compatible wire protocol: DTOs, projection and the keyed `init` map.
//!
//! Message shapes are locked in `docs/protocol.md`. Projection is pure: it
//! reads the domain state and an [`AssetCatalog`], and formats URLs through an
//! [`AssetUrlResolver`]. Only the data-URL resolver may perform I/O, and only
//! through a caller-supplied loader.

mod cache;
mod catalog;
mod dto;
mod error;
mod projection;
mod url;

pub use cache::SectionCache;
pub use catalog::AssetCatalog;
pub use dto::{
    AUTO_OVERLAY_SECTION, AutoOverlayDto, CasterDto, CastersDto, HeroBansDto, IntervieweeDto,
    MapDto, MapPoolGroupDto, MapPoolMapDto, Message, PlayerDto, ScoreboardDto, ShowMessage,
    SideIconsDto, TeamDto, TimerActionDto, TimerActionKind, TimerPayload, TimerSettingsDto,
    TimerTickDto,
};
pub use error::ProtocolError;
pub use projection::{
    DEFAULT_WATERMARK_NAME, Projector, SCOREBOARD_TYPES, auto_overlay, scores, show, timer_action,
    timer_tick,
};
pub use url::{
    AssetUrlResolver, DataUrlResolver, ServedAssetResolver, data_url_for, font_url,
    mime_for_extension,
};

pub use fgc_core::APP_NAME;

/// Default overlay server port.
pub const DEFAULT_PORT: u16 = 3001;

/// Default loopback base URL for served assets and overlays.
#[must_use]
pub fn default_base_url() -> String {
    format!("http://127.0.0.1:{DEFAULT_PORT}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_base_url_uses_default_port() {
        assert_eq!(default_base_url(), "http://127.0.0.1:3001");
    }
}
