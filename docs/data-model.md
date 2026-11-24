# Data model & persistence — Phase 2 design

Status: **approved**. Implements Phase 2 Q1 (clean domain model + conversion layer) and Q4 (versioned JSON behind a `Store` trait), with the Phase 5 content-addressed asset store applied.

---

## 1. Principles

1. **One domain model.** Idiomatic Rust, no legacy field names, no dead fields.
2. **One wire module.** The overlay protocol is produced by dedicated DTOs in a `protocol` crate. camelCase, legacy names and legacy nullability exist *only* there.
3. **Persistence is the domain, not the wire.** `app.json` serialises the domain model with `schemaVersion`.
4. **No absolute paths.** Assets are referenced by `AssetId` and identified by a content hash; URLs are derived at the wire boundary. This is what makes exports portable (C10).
5. **Derived values are computed.** Score, current map name/type and map images are derived from state + assets on every projection, never cached in the model.
6. **IDs are opaque strings** (UUID v4), preserved verbatim in DTOs so overlays and user data never see a Rust-specific format.

---

## 2. Domain types

Sketch, not final syntax. `serde` derives apply to the persistence representation only.

```rust
// ---------- identity ----------
pub struct TeamId(pub String);
pub struct PlayerId(pub String);
pub struct MatchId(pub String);
pub struct TalentId(pub String);
pub struct TournamentId(pub String);
pub struct AssetId(pub String);

// ---------- teams ----------
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub tag: String,
    pub record: Option<String>,      // wire default "0-0" when None
    pub color: String,               // "#RRGGBB"
    pub text_color: String,
    pub aux_color: String,
    pub logo: Option<AssetId>,
    pub players: Vec<Player>,
}

pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub role: Role,                  // Tank | Damage | Support
    pub position: Position,          // Core | Sub | Manager | Coach
    pub portrait: Option<AssetId>,
    pub featured_hero: Option<AssetId>,
    pub pronouns: Option<String>,
}

// ---------- match ----------
pub struct Match {
    pub id: MatchId,
    pub title: String,
    pub subtitle: String,
    pub team_a: Option<TeamId>,
    pub team_b: Option<TeamId>,
    pub maps: Vec<MapSlot>,
    pub hero_bans: Vec<MapHeroBans>,
    pub map_pool: Vec<String>,       // map *names*, matching the original
    pub first_to: u8,                // 1..=4 in the UI
    pub status: MatchStatus,         // Pre | Live | Post
    pub current_map: Option<usize>,  // 0-based
    pub swap_sides: bool,
    pub scoring: Scoring,            // Derived | Manual { a, b }
    pub side: Side,                  // Neutral(0) | HomeAttacks(1) | HomeDefends(2)
    pub talent: MatchTalent,
    pub loser_picks: bool,
    pub show_player_portraits: bool,
}

pub struct MapSlot {
    pub name: String,
    pub game_mode_name: String,      // wire field `type`; free text ("Control", "TBC", …)
    pub game_mode_manual: bool,      // true after the owner types a mode
    pub user_added: bool,            // true after + Add map
    pub completed: bool,
    pub score_a: u16,
    pub score_b: u16,
    pub chosen_by: Option<TeamSide>, // A | B
}

pub struct MapHeroBans {
    pub first_ban_team: Option<TeamSide>,
    pub team_a: Vec<String>,         // hero names
    pub team_b: Vec<String>,
}

// ---------- talent ----------
pub struct Talent {
    pub id: TalentId,
    pub display_name: String,
    pub real_name: Option<String>,
    pub pronouns: Option<String>,
    pub social: Option<SocialLink>,  // platform enum + handle
    pub portrait: Option<AssetId>,
}

pub struct MatchTalent {
    pub caster1: Option<TalentRef>,
    pub caster2: Option<TalentRef>,
    pub panel1: Option<TalentRef>,
    pub panel2: Option<TalentRef>,
    pub panel3: Option<TalentRef>,
    pub interviewer: Option<TalentRef>,
    pub interviewee: Option<IntervieweeRef>, // talent, player, or custom source
}

// ---------- assets ----------
pub enum AssetCategory { Logo, Portrait, Map, Hero, GameMode, Role, Side }

pub enum AssetAttachmentKind { HeroBanIcon, HeroTransparent, HeroBadge }

pub struct AssetAttachment {
    pub owner: AssetId,              // asset that owns this file
    pub kind: AssetAttachmentKind,   // role for the owner
}

pub struct Asset {
    pub id: AssetId,                 // logical identity referenced by teams/players
    pub name: String,                // display name
    pub category: AssetCategory,
    pub hash: String,                // 64 lowercase hex chars (SHA-256 of file bytes)
    pub ext: String,                 // png | jpg | jpeg | webp | gif | svg
    pub origin: AssetOrigin,         // Bundled | User
    pub original_filename: Option<String>,
    pub added_at: Option<String>,
    // Optional per-category extras kept only where used:
    pub icon: Option<AssetId>,       // hero → attached ban icon
    pub portrait: Option<AssetId>,   // hero portrait
    pub transparent: Option<AssetId>, // hero → transparent cut-out art
    pub badge: Option<AssetId>,       // hero → hexagonal badge art
    pub role: Option<Role>,          // hero → role (tank | damage | support)
    pub game_mode: Option<AssetId>,  // map → game mode
    pub attachment: Option<AssetAttachment>, // hidden media owned by another asset
}

// ---------- tournament ----------
// Detailed bracket semantics land in Phase 4; the shape is fixed here.
pub struct Tournament {
    pub id: TournamentId,
    pub name: String,
    pub format: BracketFormat,       // SingleElim | DoubleElim | RoundRobin | MultiStage
    pub entrants: Vec<Entrant>,      // { team_id, seed }
    pub winners: Vec<Round>,
    pub losers: Vec<Round>,
    pub grand_final: Vec<Round>,
    pub groups: Vec<Group>,
    pub group_matches: Vec<GroupMatch>,
    pub standings: BTreeMap<GroupId, Vec<Standing>>, // derived, not persisted
    pub tiebreakers: Vec<Tiebreaker>,
    pub playoff_qualifiers: u8,      // top N per group, default 2
    pub playoff_format: BracketFormat,
    pub champion: Option<TeamId>,
    pub champion_manual: bool,
    pub status: TournamentStatus,    // Setup | InProgress | Completed
    pub default_first_to: u8,
    pub grand_finals_reset: bool,
}

pub struct BracketMatch {
    pub id: MatchId,
    pub round: u32,
    pub number: u32,
    pub slot1: Option<TeamId>,
    pub slot2: Option<TeamId>,
    pub slot1_manual: bool,
    pub slot2_manual: bool,
    pub score1: u16,
    pub score2: u16,
    pub winner: Option<TeamId>,
    pub winner_manual: bool,
    pub status: BracketMatchStatus,  // Pending | Live | Completed
    pub next: Option<MatchLink>,     // { match, slot }
    pub loser_next: Option<MatchLink>,
    pub first_to: u8,
}

pub struct GroupMatch {
    pub id: MatchId,
    pub group: GroupId,
    pub team1: Option<TeamId>,
    pub team2: Option<TeamId>,
    pub score1: u16,
    pub score2: u16,
    pub status: BracketMatchStatus,   // Pending | Live | Completed
    pub first_to: u8,                 // 1..=4 in the UI
    pub winner: Option<TeamId>,       // persisted manual override
    pub winner_manual: bool,          // true when winner overrides the scores
}

// ---------- app ----------
pub struct AppState {
    pub teams: Vec<Team>,
    pub talents: Vec<Talent>,
    pub current_match: Option<Match>,
    pub tournaments: Vec<Tournament>,
    pub assets: Vec<Asset>,          // user assets; bundled metadata comes from the embedded manifest
    pub settings: Settings,
}

pub struct Settings {
    pub overlay: OverlaySettings,    // port: u16 = 3001, allow_lan: bool = false
    pub display: DisplaySettings,    // team_colors_enabled, tournament_logo, watermark_logo
    pub timer: TimerSettings,        // mode, alignment, leading_zeroes, standalone
}
```

`IntervieweeRef` has five serde-default fields: `talent_id`, `player_id`,
`custom_name`, `custom_subtext` and `custom_portrait`. Only one source is active
at a time, and the Match Controller setters clear the other fields when the
source changes. A talent reference uses the Talent library portrait and pronouns.
A player reference stores only the player id; the projection resolves the
portrait and subtext from the roster every time, so roster edits appear without
rewriting the saved reference. A
custom reference stores the name and subtext directly and points `custom_portrait`
at an asset in the Portrait category. Deleting a player clears any player
interviewee reference that pointed at it. The server projects all three shapes
into the same `IntervieweeDto` used by the overlays.

### Deliberate removals (C12)

| Removed | Reason |
|---|---|
| `matches: Match[]` | Never read or written by the UI. Match history is post-v1. |
| `Match.scoreA` / `scoreB` | Reducer wrote fields absent from the type; scores are derived or manual. |
| `substitutions[]` | No UI, no overlay consumer. Backlog item. |
| `ndi` | NDI was never implemented. |
| `Tournament.rounds` | Superseded by `winners` / `losers` / `grand_final`. |
| `MapSlot.gameModeAssetId` / `imageAssetId` | Never read; map images resolve by map *name* against the asset library. |
| `AssetEntry.path` (absolute) | Replaced by content hash + extension + origin. |
| `overlayServer.running` | Runtime-only; never persisted. |
| `currentTournament` | Redundant with `tournaments` + selection UI state. |

### Notes

- `map_pool` stays a list of **names**, because the overlay receives names and the original resolves assets by case-insensitive name match. Switching to IDs would be a silent behaviour change.
- `MapSlot.game_mode_name` is the wire `type` string; it is not an enum because the UI allows free text and the overlay renders it verbatim.
- `MapSlot.game_mode_manual` (`gameModeManual` on the wire, `#[serde(default)]`) records a mode typed by the owner. Game-mode changes skip these slots; changing the map name derives a fresh mode from the map's game mode and clears the flag. Older state files load with the flag set to `false`.
- `MapSlot.user_added` (`userAdded` on the wire, `#[serde(default)]`) records a slot created with **+ Add map**. It keeps an otherwise empty tiebreaker from being trimmed on load. Older state files load with the flag set to `false`.
- `Asset.game_mode` is the map's game mode and is edited inline on the map card. `Asset.icon` points at a hero's ban icon and is edited from the hero's **Properties…** dialog.
- `Asset.transparent` (`transparent` on the wire, `#[serde(default)]`) points at a transparent cut-out of a hero, used by the hero bans overlay. It is uploaded from the hero's **Properties…** dialog and falls back to the regular hero art when it is missing.
- `Asset.role` (`role` on the wire, `#[serde(default)]`) is the hero's role, one of `tank`, `damage` or `support`. It is picked from the hero's **Properties…** dialog and drawn as a hexagon on the hero bans overlay.
- `Asset.badge` (`badge` on the wire, `#[serde(default)]`) points at a small hexagonal emblem for a hero. It is uploaded from the hero's **Properties…** dialog and is empty for a hero without one.
- `Asset.attachment` (`attachment` on the wire, `#[serde(default)]`) marks media owned by another asset. `kind: "heroBanIcon"` is the first variant, `kind: "heroTransparent"` the second and `kind: "heroBadge"` the third; attached files are hidden from the library and pickers and are deleted with their owner. No schema version change was needed.
- `Team.record` is `Option<String>` internally; the wire DTO substitutes `"0-0"` exactly as the original does.
- Timer state (running/seconds) is **ephemeral runtime state**, not persisted — matching the original. Only timer *settings* persist.
- Asset lookup is case-insensitive by `name` for maps/heroes/game modes, matching the original `find` calls.
- Assets are content-addressed: the URL is derived from `hash` + `ext`, so replacing an image produces a new URL and cannot serve a stale cache. Full asset semantics are in `docs/assets.md`.
- Tournament `standings` are derived from `group_matches` on every change and are **not** persisted; recomputing on load avoids stale tables. Full tournament semantics are in `docs/tournament.md`.
- Whether a tournament is locked after generation is derived from its existing groups, fixtures and bracket; there is no separate persisted generation flag and `SCHEMA_VERSION` remains `2`. The generation preview, provisional bracket placements, seed edits, drag ghost and drop highlights are ephemeral UI state and are never written to `app.json`.
- Stage reset is a mutation of the existing tournament fields. It clears the selected groups or bracket and any derived champion, but adds no persisted reset or regeneration marker.

---

## 3. Persistence schema v2

`%APPDATA%/<app-id>/data/app.json` (exact app-id decided in Phase 3).

```json
{
  "schemaVersion": 2,
  "teams": [
    {
      "id": "…uuid…",
      "name": "Team Name",
      "tag": "TAG",
      "record": "0-0",
      "color": "#ff0000",
      "textColor": "#ffffff",
      "auxColor": "#090909",
      "logo": "…asset uuid…",
      "players": [
        {
          "id": "…uuid…",
          "name": "Player",
          "role": "tank",
          "position": "core",
          "portrait": null,
          "featuredHero": null,
          "pronouns": null
        }
      ]
    }
  ],
  "talents": [],
  "currentMatch": {
    "id": "…uuid…",
    "title": "",
    "subtitle": "",
    "teamA": null,
    "teamB": null,
    "maps": [
      {
        "name": "Ilios",
        "gameModeName": "Control",
        "gameModeManual": false,
        "userAdded": false,
        "completed": false,
        "scoreA": 0,
        "scoreB": 0,
        "chosenBy": null
      }
    ],
    "heroBans": [],
    "mapPool": [],
    "firstTo": 3,
    "status": "pre",
    "currentMap": null,
    "swapSides": false,
    "scoring": { "mode": "derived" },
    "side": 0,
    "talent": {},
    "loserPicks": false,
    "showPlayerPortraits": true
  },
  "tournaments": [],
  "assets": [
    {
      "id": "…uuid…",
      "name": "Peach",
      "category": "logo",
      "hash": "3fa1c9d2e4b5a6978877665544332211ffeeddccbbaa99887766554433221100",
      "ext": "png",
      "origin": "user",
      "originalFilename": "peach.png",
      "addedAt": "2026-11-06T12:00:00Z"
    }
  ],
  "settings": {
    "overlay": { "port": 3001, "allowLan": false },
    "display": {
      "teamColorsEnabled": false,
      "teamAbbreviations": false,
      "theme": "nightBroadcast",
      "tournamentLogo": null,
      "watermarkLogo": null
    },
    "timer": {
      "mode": "countup",
      "alignment": "center",
      "leadingZeroes": true,
      "showHours": true,
      "standalone": false
    }
  }
}
```

- Bundled assets are **not** persisted; their metadata comes from the embedded, build-time hashed manifest and is merged with user assets in memory.
- User asset blobs live at `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\assets\blobs\<first-two-hex>\<sha256>.<ext>`; `app.json` stores only metadata.
- `app.json.bak` holds the previous good file. Writes are temp-file + rename.
- `schemaVersion` newer than the binary is refused with a readable error and a "back up and continue" path rather than silent data loss.
- `SCHEMA_VERSION` is 2. Migration 0→1 adds the version field; migration 1→2 removes `talents[].type` and writes `role` on every occupied match talent assignment. Legacy `panelist`/`observer` values map to `analyst`/`other`; otherwise the slot supplies a default (`caster1`/`caster2` → caster, panel slots → analyst, interviewer → interviewer).
- `settings.timer.showHours` defaults to `true`, so older state files keep showing hours. When `false`, `timer.html` adds the hours to the minutes (`01:12:00` → `72:00`) while the wire tick still carries the real components.
- `settings.display.teamAbbreviations` defaults to `false`, so existing broadcasts keep the full team name. When `true`, the game overlay shows the team's short tag instead.
- `settings.display.theme` is `nightBroadcast` or `light` and defaults to `nightBroadcast`, so an older state file keeps the dark theme.
- `settings.display.themeTokens` holds the imported theme overrides as a map of CSS custom property names to values. It is omitted when empty. The values are layered over the selected preset's tokens.
- A talent assignment is `{ "talentId": "…uuid…", "role": "caster" }`. The role belongs to the assignment, not the person, so the same talent can have a different role in another match. Accepted role slugs are `caster`, `host`, `analyst`, `interviewer` and `other`.

---

## 4. Validation & repair on load

| Rule | Behaviour |
|---|---|
| Duplicate IDs | Regenerate the later duplicate, log a warning |
| `teamA == teamB` | Keep team A, clear team B |
| `currentMap` out of range | Clamp to `None` |
| `firstTo` outside 1..=4 | Clamp to 3 |
| Trailing map slots beyond the first-to format | Empty slots are trimmed to the format for `firstTo` (1/3/5/7), logged as `RepairKind::MapCountNormalized`; saved projects with fewer slots are not padded, and entered maps, tiebreakers and slots added with **+ Add map** are kept. A derived game mode on an unnamed slot is not user data; a slot added with **+ Add map** carries `userAdded: true` so it survives even while empty. |
| Map scores negative | Clamp to 0 |
| `mapPool` names with no matching asset | Kept (the overlay can still render the name); no error |
| Asset `hash` not exactly 64 lowercase hex chars | Rejected; the asset is dropped from the library and logged (C11) |
| Asset `ext` not on the allow-list | Rejected; the asset is dropped from the library and logged (C11) |
| Two assets share a hash | Allowed; identical bytes are stored once and both metadata entries remain valid |
| Unknown enum values | Rejected with a field path in the error |
| Unknown JSON fields | Top-level `AppState` keys are preserved in a `#[serde(flatten)] extra` map; nested unknown fields are ignored |
| Team name longer than 32 characters | Truncated to 32 characters, logged as `RepairKind::NameTooLong` |
| Player name longer than 24 characters | Truncated to 24 characters, logged as `RepairKind::NameTooLong` |
| Talent name longer than 24 characters | Truncated to 24 characters, logged as `RepairKind::NameTooLong` |

Import/export uses the same validator, so a hand-edited file cannot crash the app (C8).

### 4.1 M1 implementation notes

- **Score clamping happens during deserialization.** Scores are `u16`, so a negative value is clamped to 0 by a field deserializer before the repair table runs. The repair table never observes a negative score.
- **Field paths in errors.** `migrate` deserializes through `serde_path_to_error`, so an invalid enum or type reports its JSON location (for example `teams[0].players[0].role`).
- **Fixed enum spellings.** Lowercase: `Role`, `Position`, `MatchStatus`, `AssetOrigin`, `BracketMatchStatus`, `TimerMode`, `TimerAlignment`, `SocialPlatform`. Kebab-case: `AssetCategory` (`game-mode`), `BracketFormat` (`single-elim`, `double-elim`, `round-robin`, `multi-stage`), `TournamentStatus`, `Tiebreaker`. `TeamSide` is `"A"`/`"B"`; `Side` is `0`/`1`/`2`; `Scoring` is tagged with `mode`.
- **Repairs are logged once.** `validate_and_repair` emits a `tracing::warn!` per repair; `migrate` calls it without logging the results again.

### 4.2 M6 reference rules

- Deleting a team is a UI operation, not a load repair. `fgc_core::team_references` reports every tournament entrant, winners/losers/grand-final slot and winner, group membership, group fixture, champion and current-match `teamA`/`teamB` reference.
- The Teams view blocks deletion while references exist and offers an explicit cleanup path. `remove_team_references` clears those references before the team is removed.
- Deleting a talent clears any current-match caster, panel or interviewer assignment that pointed at them; talents are not referenced by tournaments.
- Reference discovery never repairs dangling IDs on load. A hand-edited file with a missing team still loads, and the overlay projection renders the slot as empty.

---

## 5. Wire boundary

- `protocol` crate owns every DTO and every `serde` rename.
- Projection functions: `scoreboard_dto(state, match, assets)`, `casters_dto(...)`, `timer_dto(...)`, `init_map(...)`.
- `AssetResolver` turns `AssetId` into a URL:
  - bundled and user → `http://<host>:<port>/assets/<hash>.<ext>`
  - fonts → `http://<host>:<port>/assets/fonts/<name>.woff2` (stable path)
  - the hash is content identity, so the category is not part of the URL.
- A feature flag `asset_urls = served | data_url` (default `served`) covers third-party overlays that expect inline images (Q3).

---

## 6. Decisions locked

1. `current_match` is a single `Option<Match>`; there is no match history in v1.
2. `map_pool` stays a list of names rather than IDs, required for wire parity.
3. Timer settings are persisted; timer run-state is ephemeral.
4. The asset library keeps bundled and user assets in one in-memory list, distinguished by `origin`, rather than seven parallel arrays.
5. Assets are content-addressed by `hash` + `ext`; `/assets/<hash>.<ext>` is the final URL and there is no `/user-assets` route in v1.
6. Bundled asset metadata comes from the embedded manifest; only user asset metadata is persisted.
