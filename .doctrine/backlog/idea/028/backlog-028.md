# IDE-028: Animated HTML stinger transitions for team and head-to-head game overlay

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner request (2026-11-21): add animated stinger transitions that incorporate
the tournament logo and team logos. The HTML stingers must be available as
separate browser sources and must also be built into the auto overlay. When
transitioning into an overlay for a respective team, show a stinger featuring
that team's logo. When transitioning to `game-overlay.html`, show a head-to-head
stinger with the two team logos. IDE-004 is a hard dependency because the auto
overlay already owns scene selection and iframe embedding; this item follows the
§C presentation roadmap umbrella in IDE-018.

Research into common esports broadcast practice found that the strongest shared
convention is a **modular system**, not a library of unrelated clips. Riot's
Worlds 2022 package used a toolkit that swapped team colours and logos late in
production and combined all 24 teams into one event flag
([Digiday](https://digiday.com/marketing/how-riot-games-developed-its-brand-identity-for-the-league-of-legends-world-championship),
[Stash Media](https://stashmedia.tv/tendril-ramps-up-riot-worlds-2022-with-branding-toolkit)).
MSI 24 used one motion device, “The Aberration”, at different intensities
([Studio Dumbar/DEPT®](https://studiodumbar.com/work/msi-24)). OWL favoured fast
2D swipe/wipe motion, layered cards, sharp corners and dry cuts rather than a
generic spinning 3D tunnel
([OWL motion review](https://medium.com/did-you-know-the-journal-blog/overwatch-leagues-motion-graphics-review-52a6df4a4522),
[OWL UI analysis](https://medium.com/@avilebroker_58270/overwatch-league-ui-pt-1-introduction-analysis-69f7e9ec3de9)).
OWWC/Open Division added ceremonial national-team pageantry
([Capacity Studios](https://capacity.gg/project/overwatch-world-cup)), while
OWCS used regional wordmarks and palettes
([Shelly Amine](https://shellyamine.com/portfolio/owcs-logos)). VCT toolkits used
shared timing/easing tokens, responsive text boxes and a perspective grid for
forward momentum
([Ian Crane](https://iancrane.xyz/projects/valorant),
[BUCK](https://buck.co/work/valorant-champions)); LCK 2021 deliberately kept
graphics minimal so the game stayed the focus
([Inven Global](https://invenglobal.com/articles/12982/the-new-era-of-lck-is-here-a-visual-journey-into-the-new-broadcast-and-lol-park)).
The LPL team-logo reveal is a useful example of many team marks inside one
system ([LPL English](https://www.youtube.com/watch?v=58P775Hozhw)).
Official league marks must not be copied without permission; placeholder data
and owner-supplied assets should drive implementation.

Proposed direction: one plain HTML/CSS/JS presentation system with separate
standalone pages (team stinger and head-to-head stinger) plus a reusable module
that `auto.html` can drive. Data should be late-bound from the existing overlay
wire state: team name/tag/logo/colours, tournament logo/name, score, format and
stage. Use shared duration/easing tokens, animate `transform` and `opacity`,
keep the document transparent at 1920×1080, and provide fallbacks for missing
logos, blank colours, long names and null teams. A team stinger should be short
(roughly 2–4s in the default flow, with a longer 4–7s variant for formal intros);
the head-to-head hold should be about 2–3s before handing off to the match bug.
Avoid long “epic” sequences, unrelated per-team styles, centre-frame logos over
gameplay, heavy filters and network/CDN dependencies.

Acceptance criteria:

- Separate HTML stinger sources can be loaded directly as OBS Browser Sources
  and replayed on demand.
- Team stinger uses the tournament logo and the selected team's logo/name from
  current state, with a safe monogram fallback.
- Game-overlay stinger shows both logos in a symmetrical head-to-head lockup
  with event context, then clears cleanly to `game-overlay.html`.
- Both stingers are available inside the auto overlay without a second Browser
  Source or duplicate hand-authored variants.
- Missing/blank team data, long names, dark/light colour collisions and reduced
  motion are handled without overflow, console errors or opaque backgrounds.
- QA uses real desktop WebView2 over CDP at 1920×1080 with DOM/computed-style
  evidence; no screenshots are published.
