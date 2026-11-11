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
4. Choose a **Team logo**, then set the **Background**, **Text** and
   **Logo background** colours. The **Preview** panel shows the result.
5. Open the **Players** tab and select **+ Add player** for each player.
6. Fill in **Player name**, **Role**, **Position**, **Portrait**, **Featured
   hero** and **Pronouns**, then select **Done**.
7. Repeat for the second team.

See [Teams and Talent](Teams-and-Talent) for the full field list.

## 2. Add your talent

1. Open **Talent**.
2. Select **+ Add talent**.
3. Fill in **Display name**, optional **Real name**, **Pronouns**, **Social
   platform**, **Handle** and **Portrait**.

Add every caster, panel member and interviewer you expect to use.

## 3. Add images

1. Open **Assets**.
2. Choose a **Category**: Logo, Portrait, Map, Hero, Game mode, Role or Side.
3. Select **Import file…** and choose an image, or **Import from URL…** and
   paste an image address.
4. Repeat for the images you need.

PNG, JPEG, WebP, GIF and SVG are accepted up to 20 MB. Identical images are
stored once. See [Assets](Assets).

## 4. Build the match

1. Open **Match Controller**.
2. Select **Create match**.
3. On the **Match** tab, set **Title**, **Subtitle**, **Format**, **Status**,
   **Team A** and **Team B**.
4. On **Map Selection**, select **+ Add map** and fill in each map. Mark the
   current map with **Current map** and finished maps with **Complete**.
5. On **Hero Bans**, choose the first-ban team and the hero each team banned.
6. On **Talent**, assign **Caster 1**, **Caster 2**, the panel slots and the
   interviewer.
7. On **Map Pool**, tick the maps that should appear and put them in order.
8. On **Timer**, set the clock defaults, then use **Start**, **Pause** and
   **Reset** as needed.
9. On **Config**, check **Team colours**, **Player portraits** and the branding
   images.

Every change is sent to the overlays immediately.

## 5. Start the overlay server

1. Open **Overlay**.
2. Select **Start server**.
3. Confirm the status reads **Running** and the **Base URL** is
   `http://127.0.0.1:3001` unless you changed the port.

## 6. Add the overlays to OBS

For each overlay you want:

1. In OBS, add a **Browser** source.
2. Set **Width** to `1920` and **Height** to `1080`.
3. Paste the URL from the **Browser sources** list. The **Copy URL** button
   copies it for you.
4. Turn off **Shutdown source when not visible** so the overlay keeps its
   state.
5. Leave the background transparent.

The seven URLs are listed on [Overlays in OBS](Overlays-in-OBS).

## 7. Check it live

1. Change the match **Subtitle** in **Match Controller**.
2. Confirm the game overlay updates without reloading.
3. Restart the server from **Overlay** and confirm the overlays reconnect.

If an overlay stays blank, see [Troubleshooting](Troubleshooting).

## Next

- Running a bracket? Continue with [Tournament](Tournament).
- Moving to another machine? See [Data and Backups](Data-and-Backups).
