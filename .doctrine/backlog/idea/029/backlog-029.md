# IDE-029: Flythrough overlay with looping video and lower-third series information

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner request (2026-11-21): add a flythrough overlay with a looping video file
and a lower-third bug carrying series information — who is playing, the current
score, the current round and ban information. The owner will share a personal
example when this is ready to build; capture that example here before design or
asset work starts. Research existing esports tournaments for inspiration. This
item follows the §C presentation roadmap umbrella in IDE-018. No hard dependency
is recorded yet; if implementation is gated on tournament/bracket work, add
`needs IDE-009` at that point.

“Flythrough” is not a formal term in the official Riot/Blizzard design sources
reviewed. The closest documented ideas are broadcast/package openers, team/player
intro animations, perspective-grid or push-through transitions, and persistent
lower-thirds/match bugs. Current packages treat motion as a way to move
information between states rather than as a decorative tunnel. Riot's Worlds
operation uses a world-feed model with clean, reusable graphics zones and
localisation-friendly layouts
([Riot, 2022](https://www.riotgames.com/en/news/riot-esports-delivering-custom-global-broadcasts-worlds-2022),
[Riot engineering, 2019](https://www.riotgames.com/en/news/engineering-esports-tech-powers-worlds)).
OWL's motion language was fast, 2D and swipe-based
([OWL motion review](https://medium.com/did-you-know-the-journal-blog/overwatch-leagues-motion-graphics-review-52a6df4a4522)),
while VCT/BUCK used a perspective grid and push-through spaces to convey forward
momentum ([BUCK](https://buck.co/work/valorant-champions)). MSI 24 and VCT
toolkits show the value of one adaptive motion motif with calm/reactive/hype
intensities and responsive text boxes
([Studio Dumbar/DEPT®](https://studiodumbar.com/work/msi-24),
[Ian Crane](https://iancrane.xyz/projects/valorant)). Community Overwatch package
documentation lists the data a match bug normally carries: event and stage, team
logos/names/colours, series and map score, map/veto state, players, casters and
fallback states
([Broadcast.gg](https://broadcast.gg/docs/modern-package-overwatch)). LCK 2021 is
the useful counterweight: keep it readable and do not distract from gameplay
([Inven Global](https://invenglobal.com/articles/12982/the-new-era-of-lck-is-here-a-visual-journey-into-the-new-broadcast-and-lol-park)).
Do not copy official league or team marks without permission.

Proposed direction: a transparent 1920×1080 HTML page with a local looping video
layer and a compact lower-third bug. Reuse the existing projection rather than
introducing a new persisted schema: team names/tags/logos, `scoreA`/`scoreB`,
current map or round, map/veto/ban state and series format. Video should be a
bundled asset with a poster/fallback still, muted autoplay, `loop`, and a
documented local-file path; no remote URLs. Keep critical text inside a 5–10%
title-safe margin and out of the centre/crosshair area. Treat the loop as a
transition or intermission layer, not a permanent gameplay element. Provide
states for no match, missing video, long team names, null teams, unknown round
and empty bans. Animate only `transform`/`opacity`; avoid heavy filters and
parallax that break alpha or readability after stream compression.

Acceptance criteria:

- The owner's example is recorded in this item before visual design starts.
- A local looping video plays muted and seamlessly in the actual capture path,
  with a still-image fallback if it cannot load.
- The lower third shows who is playing, current score, current round/map and ban
  information from the existing overlay state, with explicit empty states.
- Long names, missing logos, null teams and missing video do not overflow or
  produce console errors.
- The page is transparent, 1920×1080, avoids the centre-frame safe area, and is
  verified in real desktop WebView2 over CDP with DOM/computed-style evidence.
- No screenshots are published and no remote assets, CDNs or official marks are
  introduced without permission.
