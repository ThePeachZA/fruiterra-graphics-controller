# Quick Start

This page takes you from a fresh install to a working overlay in OBS.

## Before you start

- The app is installed, or the portable build is extracted.
- OBS Studio is installed.
- You have the team logos and talent portraits you want to use.

## 1. Add your teams

1. Open **Teams** in the sidebar.
2. Select **+ New team**.
3. On **Team Info**, fill in **Team name**, **Team tag** (3–4 letters) and
   **Record**.
4. Choose a **Team logo**. The app samples the team's colours from the logo;
   adjust **Background**, **Text** and **Logo background** if needed. Turn on
   **Separate logo background** only if the logo needs its own background
   colour. The **Preview** panel shows the result.
5. Open the **Players** tab and select **+ Add player** for each player.
6. Fill in **Player name**, **Role**, **Position**, **Portrait**, **Featured
   hero** and **Pronouns**, then select **Done**.
7. Repeat for the second team.

See [Teams and Talent](Teams-and-Talent) for the full field list.

## 2. Add your talent

1. Open **Talent**.
2. Select **+ Add talent**.
3. Fill in **Display name**, optional **Real name**, **Type**, **Pronouns**,
   **Social** and **Portrait**.
4. Repeat for each caster or panel member.

## 3. Import assets

1. Open **Assets**.
2. Choose a **Category** (for example **Logo** or **Portrait**).
3. Select **Import files…** or **Import from URL…**.

Imported images become available in the logo, portrait, hero and map pickers.
You can also upload an image directly from many pickers.

See [Assets](Assets) for categories, limits and SVG rules.

## 4. Create the match

1. Open **Match Controller**.
2. If the page shows **No current match**, select **Create match**.
3. On the **Match** tab, set **Title**, **Subtitle**, optional **Scheduled
   start**, **Format** and **Status**.
4. Choose **Team A** and **Team B**.
5. Open **Maps & Bans**, select the map for each slot and mark the **Current
   map**.
6. Open **Talent** and assign **Caster 1**, **Caster 2** and any panel slots.
   Set each slot's **Role**; there is no role on the talent library record.
7. Use the timer widget above the status bar. Open **Timer settings** to choose
   **Count up** or **Count down**, then select the icon **Start** when you go
   live. The widget stays available on every page.
8. Open **Config** to set team colours, player portraits, loser picks, hero
   bans and branding.

## 5. Start the overlay server

1. Open **Overlay** in the sidebar. If **Start server on launch** is enabled in
   Settings, the server may already be running.
2. If needed, select **Start server**.
3. Note the **Base URL** (default `http://127.0.0.1:3001`). Select **Show
   URLs** if the Browser Source URLs are hidden.

## 6. Add Browser Sources in OBS

Create one Browser Source per overlay you want to show:

1. In OBS, add a **Browser** source.
2. Set **Width** to `1920` and **Height** to `1080`.
3. Paste the overlay URL from the **Overlay** page.
4. Turn **off** **Shutdown source when not visible**.
5. Leave the background transparent.

The usual first set is **Game overlay**, **Map select** and **Casters row**.
See [Overlays in OBS](Overlays-in-OBS) for every URL and the recommended source
settings.

## 7. Go live

- Change the score, map, bans and talent in the Match Controller. The overlays
  update immediately.
- Use **Force update** if an overlay looks out of date.
- When the match ends, set **Status** to **Complete**.

## Next steps

- [Tournament](Tournament) — build a bracket or round-robin event.
- [Settings](Settings) — change the port, start the server automatically, or
  allow LAN connections.
- [Remote overlays](Remote-Overlays) — post-v1 Tailscale URLs, firewall, teardown
  and troubleshooting.
- [Troubleshooting](Troubleshooting) — fix a blank overlay, port conflict or
  SmartScreen warning.
