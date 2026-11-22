use std::collections::{BTreeMap, BTreeSet};

use fgc_core::{
    AppState, AssetCategory, AssetId, Match, Scoring, Team, TeamId, TeamSide, TimerSettings,
};

use crate::{
    AUTO_OVERLAY_SECTION, AssetCatalog, AssetUrlResolver, AutoOverlayDto, CasterDto, CastersDto,
    HeroBansDto, IntervieweeDto, MapDto, MapPoolGroupDto, MapPoolMapDto, Message, PlayerDto,
    ScoreboardDto, ShowMessage, SideIconsDto, TeamDto, TimerActionDto, TimerPayload,
    TimerSettingsDto, TimerTickDto,
};

/// Message types that carry the identical scoreboard payload.
pub const SCOREBOARD_TYPES: [&str; 5] = [
    "scoreboard",
    "gameOverlay",
    "mapSelect",
    "heroBans",
    "teamRoster",
];

/// Bundled logo used as the watermark when no custom watermark is configured.
pub const DEFAULT_WATERMARK_NAME: &str = "Peach";

/// Projects domain state into the wire DTOs.
///
/// Projection is pure in served mode: it reads the catalogue and formats URLs,
/// but never touches the filesystem.
pub struct Projector<'a> {
    state: &'a AppState,
    catalog: &'a AssetCatalog,
    resolver: &'a dyn AssetUrlResolver,
}

impl<'a> Projector<'a> {
    /// Creates a projector over the current state, asset catalogue and resolver.
    #[must_use]
    pub fn new(
        state: &'a AppState,
        catalog: &'a AssetCatalog,
        resolver: &'a dyn AssetUrlResolver,
    ) -> Self {
        Self {
            state,
            catalog,
            resolver,
        }
    }

    /// Builds the scoreboard payload.
    #[must_use]
    pub fn scoreboard(&self, match_state: &Match) -> ScoreboardDto {
        let team_a = self.team(match_state.team_a.as_ref());
        let team_b = self.team(match_state.team_b.as_ref());

        let logo_a = team_a
            .and_then(|team| team.logo.as_ref())
            .and_then(|id| self.resolve(id));
        let logo_b = team_b
            .and_then(|team| team.logo.as_ref())
            .and_then(|id| self.resolve(id));

        let (score_a, score_b) = scores(match_state);
        let current = match_state
            .current_map
            .and_then(|index| match_state.maps.get(index));
        let casters = self.casters(match_state);
        let show_hero_bans = self.state.settings.display.show_hero_bans;

        ScoreboardDto {
            team_a: team_a.map(|team| team_dto(team, logo_a.clone())),
            team_b: team_b.map(|team| team_dto(team, logo_b.clone())),
            score_a,
            score_b,
            title: match_state.title.clone(),
            subtitle: match_state.subtitle.clone(),
            current_map: current.map(|slot| slot.name.clone()).unwrap_or_default(),
            current_map_index: match_state.current_map,
            first_to: match_state.first_to,
            map_type: current
                .map(|slot| slot.game_mode_name.clone())
                .unwrap_or_default(),
            status: match_state.status,
            side: match_state.side,
            swap_sides: match_state.swap_sides,
            loser_picks: match_state.loser_picks,
            game_mode_icons: self.game_mode_icons(),
            hero_icons: if show_hero_bans {
                self.hero_icons()
            } else {
                BTreeMap::new()
            },
            hero_portraits: self.hero_portraits(match_state),
            side_icons: self.side_icons(),
            role_icons: self.role_icons(),
            team_a_players: self.players(team_a),
            team_b_players: self.players(team_b),
            maps: self.maps(match_state, team_a, team_b),
            hero_bans: if show_hero_bans {
                hero_bans(match_state)
            } else {
                Vec::new()
            },
            map_pool: match_state.map_pool.clone(),
            map_pool_data: self.map_pool_data(match_state, logo_a.as_deref(), logo_b.as_deref()),
            casters_data: casters.casters_data,
            tournament_logo_url: self
                .state
                .settings
                .display
                .tournament_logo
                .as_ref()
                .and_then(|id| self.resolve(id)),
            watermark_logo_url: self.watermark_logo_url(),
            team_colors_enabled: self.state.settings.display.team_colors_enabled,
            show_player_portraits: match_state.show_player_portraits,
        }
    }

    /// Builds all five scoreboard-bearing messages with identical payloads.
    #[must_use]
    pub fn scoreboard_messages(&self, match_state: &Match) -> Vec<Message<ScoreboardDto>> {
        let data = self.scoreboard(match_state);
        SCOREBOARD_TYPES
            .iter()
            .map(|kind| Message::new(*kind, data.clone()))
            .collect()
    }

    /// Builds the `casters` payload.
    #[must_use]
    pub fn casters(&self, match_state: &Match) -> CastersDto {
        let slots = [
            match_state.talent.caster1.as_ref(),
            match_state.talent.caster2.as_ref(),
            match_state.talent.panel1.as_ref(),
            match_state.talent.panel2.as_ref(),
            match_state.talent.panel3.as_ref(),
            match_state.talent.interviewer.as_ref(),
        ];

        let casters_data = slots
            .into_iter()
            .flatten()
            .filter_map(|slot| {
                self.state
                    .talents
                    .iter()
                    .find(|talent| talent.id == slot.talent_id)
                    .map(|talent| CasterDto {
                        role: slot.role,
                        display_name: talent.display_name.clone(),
                        pronouns: talent.pronouns.clone(),
                        social_media: talent.social.clone(),
                        portrait_url: talent.portrait.as_ref().and_then(|id| self.resolve(id)),
                    })
            })
            .collect();

        CastersDto {
            casters_data,
            interviewee: self.interviewee(match_state),
        }
    }

    /// Builds the interview subject, from the talent library or a custom entry.
    #[must_use]
    fn interviewee(&self, match_state: &Match) -> Option<IntervieweeDto> {
        let interviewee = match_state.talent.interviewee.as_ref()?;
        let talent = interviewee
            .talent_id
            .as_ref()
            .and_then(|id| self.state.talents.iter().find(|talent| &talent.id == id));

        let (display_name, subtext) = match talent {
            Some(talent) => (
                talent.display_name.clone(),
                talent
                    .pronouns
                    .clone()
                    .or_else(|| interviewee.custom_subtext.clone()),
            ),
            None => (
                interviewee.custom_name.clone()?,
                interviewee.custom_subtext.clone(),
            ),
        };

        if display_name.trim().is_empty() {
            return None;
        }

        Some(IntervieweeDto {
            display_name,
            subtext: subtext.filter(|text| !text.trim().is_empty()),
            portrait_url: talent
                .and_then(|talent| talent.portrait.as_ref())
                .and_then(|id| self.resolve(id)),
        })
    }

    /// Builds the `casters` message.
    #[must_use]
    pub fn casters_message(&self, match_state: &Match) -> Message<CastersDto> {
        Message::new("casters", self.casters(match_state))
    }

    fn team(&self, id: Option<&TeamId>) -> Option<&Team> {
        let id = id?;
        self.state.teams.iter().find(|team| &team.id == id)
    }

    fn resolve(&self, id: &AssetId) -> Option<String> {
        self.catalog
            .get(id)
            .and_then(|asset| self.resolver.url(asset))
    }

    /// Resolves the large art for a hero asset.
    ///
    /// A hero asset's own file is its portrait art; an optional `portrait` link
    /// overrides it. This mirrors the original's `hero.path || hero.portraitPath`
    /// preference for the large render.
    fn hero_art_url(&self, asset: &fgc_core::Asset) -> Option<String> {
        asset
            .portrait
            .as_ref()
            .and_then(|id| self.catalog.get(id))
            .and_then(|portrait| self.resolver.url(portrait))
            .or_else(|| self.resolver.url(asset))
    }

    /// Watermark URL, falling back to the bundled `Peach` logo.
    ///
    /// The fallback only applies when no watermark is configured or the
    /// configured asset is missing; a configured asset whose bytes are
    /// unavailable resolves to `null`, matching the original.
    fn watermark_logo_url(&self) -> Option<String> {
        match self.state.settings.display.watermark_logo.as_ref() {
            Some(id) => match self.catalog.get(id) {
                Some(asset) => self.resolver.url(asset),
                None => self.default_watermark_url(),
            },
            None => self.default_watermark_url(),
        }
    }

    fn default_watermark_url(&self) -> Option<String> {
        self.catalog
            .find_exact(AssetCategory::Logo, DEFAULT_WATERMARK_NAME)
            .and_then(|asset| self.resolver.url(asset))
    }

    fn players(&self, team: Option<&Team>) -> Vec<PlayerDto> {
        let Some(team) = team else {
            return Vec::new();
        };

        team.players
            .iter()
            .map(|player| PlayerDto {
                name: player.name.clone(),
                role: player.role,
                position: player.position,
                pronouns: player.pronouns.clone(),
                portrait_url: player.portrait.as_ref().and_then(|id| self.resolve(id)),
                hero_image_url: player
                    .featured_hero
                    .as_ref()
                    .and_then(|id| self.catalog.get(id))
                    .and_then(|asset| self.hero_art_url(asset)),
            })
            .collect()
    }

    fn maps(
        &self,
        match_state: &Match,
        team_a: Option<&Team>,
        team_b: Option<&Team>,
    ) -> Vec<MapDto> {
        match_state
            .maps
            .iter()
            .map(|slot| {
                let chosen_by_name = match slot.chosen_by {
                    Some(TeamSide::A) => team_a.map(|team| team.tag.clone()),
                    Some(TeamSide::B) => team_b.map(|team| team.tag.clone()),
                    None => None,
                };

                MapDto {
                    name: slot.name.clone(),
                    map_type: slot.game_mode_name.clone(),
                    completed: slot.completed,
                    score_a: slot.score_a,
                    score_b: slot.score_b,
                    map_image_url: self
                        .catalog
                        .find_by_name(AssetCategory::Map, &slot.name)
                        .and_then(|asset| self.resolver.url(asset)),
                    chosen_by: slot.chosen_by,
                    chosen_by_name,
                }
            })
            .collect()
    }

    fn map_pool_data(
        &self,
        match_state: &Match,
        logo_a: Option<&str>,
        logo_b: Option<&str>,
    ) -> Vec<MapPoolGroupDto> {
        if match_state.map_pool.is_empty() {
            return Vec::new();
        }

        let mut chosen_by = BTreeMap::new();
        for slot in &match_state.maps {
            if !slot.name.is_empty() {
                chosen_by.insert(slot.name.clone(), slot.chosen_by);
            }
        }

        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        for name in &match_state.map_pool {
            let Some(asset) = self.catalog.find_by_name(AssetCategory::Map, name) else {
                continue;
            };
            let mode_name = asset
                .game_mode
                .as_ref()
                .and_then(|id| self.catalog.get(id))
                .map(|mode| mode.name.clone())
                .unwrap_or_else(|| "Other".to_owned());

            match groups.iter_mut().find(|(mode, _)| mode == &mode_name) {
                Some((_, maps)) => maps.push(name.clone()),
                None => groups.push((mode_name, vec![name.clone()])),
            }
        }

        let mut data: Vec<MapPoolGroupDto> = groups
            .into_iter()
            .map(|(mode_name, names)| {
                let mode_icons = self
                    .catalog
                    .find_exact(AssetCategory::GameMode, &mode_name)
                    .and_then(|asset| self.resolver.url(asset))
                    .into_iter()
                    .collect();

                let maps = names
                    .iter()
                    .map(|name| {
                        let chosen = chosen_by.get(name).copied().flatten();
                        let chosen_by_logo = match chosen {
                            Some(TeamSide::A) => logo_a.map(ToOwned::to_owned),
                            Some(TeamSide::B) => logo_b.map(ToOwned::to_owned),
                            None => None,
                        };

                        MapPoolMapDto {
                            name: name.clone(),
                            image_url: self
                                .catalog
                                .find_by_name(AssetCategory::Map, name)
                                .and_then(|asset| self.resolver.url(asset)),
                            chosen_by: chosen,
                            chosen_by_logo,
                        }
                    })
                    .collect();

                MapPoolGroupDto {
                    mode_name,
                    mode_icons,
                    maps,
                }
            })
            .collect();

        data.sort_by(|left, right| {
            mode_sort_key(&left.mode_name).cmp(&mode_sort_key(&right.mode_name))
        });
        data
    }

    fn game_mode_icons(&self) -> BTreeMap<String, String> {
        let mut icons = BTreeMap::new();
        for asset in self.catalog.in_category(AssetCategory::GameMode) {
            if let Some(url) = self.resolver.url(asset) {
                icons.insert(asset.name.to_lowercase(), url);
            }
        }
        icons
    }

    /// IDs of assets used as another hero's ban icon.
    fn icon_asset_ids(&self) -> BTreeSet<&str> {
        self.catalog
            .in_category(AssetCategory::Hero)
            .filter(|asset| asset.is_attachment())
            .map(|asset| asset.id.as_str())
            .chain(
                self.catalog
                    .in_category(AssetCategory::Hero)
                    .filter_map(|asset| asset.icon.as_ref())
                    .map(AssetId::as_str),
            )
            .collect()
    }

    /// Content identities that belong to a ban icon rather than hero art.
    ///
    /// A duplicate hero asset can carry the same bytes as an icon. Without
    /// this set it wins the name-keyed portrait map and `hero-bans.html` shows
    /// the small ban image instead of the hero's art.
    fn icon_identities(&self) -> BTreeSet<(String, String)> {
        let mut identities = BTreeSet::new();
        for id in self.icon_asset_ids() {
            if let Some(asset) = self
                .catalog
                .in_category(AssetCategory::Hero)
                .find(|asset| asset.id.as_str() == id)
            {
                identities.insert((asset.hash.clone(), asset.ext.clone()));
            }
        }
        identities
    }

    fn hero_icons(&self) -> BTreeMap<String, String> {
        let icon_ids = self.icon_asset_ids();
        let icon_identities = self.icon_identities();

        let mut icons = BTreeMap::new();
        for asset in self.catalog.in_category(AssetCategory::Hero) {
            // Attached icons, legacy linked icons and duplicate icon files are
            // not selectable heroes. The owning hero resolves the icon below.
            if icon_ids.contains(asset.id.as_str())
                || icon_identities.contains(&(asset.hash.clone(), asset.ext.clone()))
            {
                continue;
            }
            if let Some(url) = self.hero_icon_url(asset) {
                icons.insert(asset.name.clone(), url);
            }
        }
        icons
    }

    /// Resolves the small icon for a hero asset.
    ///
    /// Prefers an explicit `icon` link, then the large art (portrait override or
    /// the asset's own file). Imported heroes usually have no icon link, so
    /// without this fallback the overlay would draw an empty ban strip.
    fn hero_icon_url(&self, asset: &fgc_core::Asset) -> Option<String> {
        asset
            .icon
            .as_ref()
            .and_then(|id| self.catalog.get(id))
            .and_then(|icon| self.resolver.url(icon))
            .or_else(|| self.hero_art_url(asset))
    }

    fn hero_portraits(&self, match_state: &Match) -> BTreeMap<String, String> {
        let banned: BTreeSet<&str> = match_state
            .hero_bans
            .iter()
            .flat_map(|bans| {
                bans.team_a
                    .iter()
                    .chain(bans.team_b.iter())
                    .map(String::as_str)
            })
            .collect();

        let icon_ids = self.icon_asset_ids();
        let icon_identities = self.icon_identities();

        let mut portraits = BTreeMap::new();
        for asset in self.catalog.in_category(AssetCategory::Hero) {
            // Ban icons are for `game-overlay.html` only. Skip attached icons,
            // legacy linked icons and duplicate files with the same bytes.
            if icon_ids.contains(asset.id.as_str())
                || icon_identities.contains(&(asset.hash.clone(), asset.ext.clone()))
            {
                continue;
            }
            if !banned.contains(asset.name.as_str()) {
                continue;
            }
            if let Some(url) = self.hero_art_url(asset) {
                portraits.insert(asset.name.clone(), url);
            }
        }
        portraits
    }

    fn side_icons(&self) -> SideIconsDto {
        let mut icons = SideIconsDto::default();
        for asset in self.catalog.in_category(AssetCategory::Side) {
            let name = asset.name.to_lowercase();
            if name.contains("attack") && icons.attack.is_none() {
                icons.attack = self.resolver.url(asset);
            } else if name.contains("defend") && icons.defend.is_none() {
                icons.defend = self.resolver.url(asset);
            }
        }
        icons
    }

    fn role_icons(&self) -> BTreeMap<String, String> {
        let mut icons = BTreeMap::new();
        for asset in self.catalog.in_category(AssetCategory::Role) {
            if let Some(url) = self.resolver.url(asset) {
                icons.insert(asset.name.to_lowercase(), url.clone());
                icons.insert(asset.name.clone(), url);
            }
        }
        icons
    }
}

/// Derived or manual scores, with draws ignored.
#[must_use]
pub fn scores(match_state: &Match) -> (u16, u16) {
    match &match_state.scoring {
        Scoring::Manual { a, b } => (*a, *b),
        Scoring::Derived => fgc_core::derived_scores(match_state),
    }
}

/// Builds a `matchTimer` tick message.
#[must_use]
pub fn timer_tick(
    settings: &TimerSettings,
    hours: u32,
    minutes: u32,
    seconds: u32,
) -> Message<TimerPayload> {
    Message::new(
        "matchTimer",
        TimerPayload::Tick(TimerTickDto {
            hours,
            minutes,
            seconds,
            settings: TimerSettingsDto::from(settings),
        }),
    )
}

/// Builds a `matchTimer` control message.
#[must_use]
pub fn timer_action(action: TimerActionDto) -> Message<TimerPayload> {
    Message::new("matchTimer", TimerPayload::Action(action))
}

/// Builds a `show` message with the top-level `visible` field overlays read.
#[must_use]
pub fn show(visible: bool) -> ShowMessage {
    ShowMessage::new(visible)
}

/// Builds the session-only `autoOverlay` scene-selection message.
#[must_use]
pub fn auto_overlay(included: Vec<String>, active: Option<String>) -> Message<AutoOverlayDto> {
    Message::new(AUTO_OVERLAY_SECTION, AutoOverlayDto::new(included, active))
}

fn team_dto(team: &Team, logo_url: Option<String>) -> TeamDto {
    let record = team
        .record
        .as_deref()
        .filter(|record| !record.is_empty())
        .unwrap_or("0-0")
        .to_owned();

    TeamDto {
        name: team.name.clone(),
        tag: team.tag.clone(),
        record,
        color: team.color.clone(),
        logo_bg_color: if !team.logo_background {
            team.color.clone()
        } else if team.aux_color.is_empty() {
            "#090909".to_owned()
        } else {
            team.aux_color.clone()
        },
        logo_url,
    }
}

fn hero_bans(match_state: &Match) -> Vec<HeroBansDto> {
    match_state
        .hero_bans
        .iter()
        .map(|bans| HeroBansDto {
            first_ban_team: bans.first_ban_team,
            team_a: bans.team_a.clone(),
            team_b: bans.team_b.clone(),
        })
        .collect()
}

fn mode_sort_key(name: &str) -> (bool, &str) {
    (matches!(name, "Flashpoint" | "Push"), name)
}
