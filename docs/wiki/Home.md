# Fruiterra Graphics Controller

Fruiterra Graphics Controller is a Windows desktop application for running an
Overwatch broadcast. It keeps your teams, talent, matches, maps, assets and
tournaments in one place, and serves the graphics that OBS displays.

The app replaces the earlier Google Sheets-based production workflow. Your
overlay HTML continues to work: the overlay server speaks the same WebSocket
protocol as before.

> [!NOTE]
> **Version 1 is Windows-only and unsigned.** The installer and portable build
> are published on the [Releases page](https://github.com/ThePeachZA/fruiterra-graphics-controller/releases)
> once the manual release checks have passed. Windows SmartScreen may warn on
> first launch.

## What you need

- Windows 10 or Windows 11, 64-bit.
- The Microsoft Edge WebView2 runtime. The installer adds it if it is missing.
- OBS Studio.
- One OBS Browser Source for each overlay you want to show.

## Five-minute setup

1. Install the app, or extract the portable ZIP and run `fruiterra-gc.exe`.
2. Open **Teams** and add the two teams, their logos and their players.
3. Open **Talent** and add the casters and panel members you will use.
4. Open **Assets** and import any logos, portraits, maps or heroes you need.
5. Open **Match Controller**, create a match, choose the teams and maps, and set
   the title, subtitle and first-to score.
6. Open **Overlay** and select **Start server**.
7. In OBS, add a Browser Source for each overlay at **1920×1080** and paste the
   matching URL from the **Browser sources** list.
8. Make a change in the app and confirm the overlay updates in OBS.

## Guide contents

| Page | What it covers |
|---|---|
| [Installation](Installation) | Installer, portable build, SmartScreen, checksums, uninstall |
| [Quick Start](Quick-Start) | A first broadcast from an empty install |
| [Overlays in OBS](Overlays-in-OBS) | The seven overlays, OBS settings, reconnect and overrides |
| [Custom overlays](Custom-Overlays) | Writing your own overlay against the live feed |
| [Teams and Talent](Teams-and-Talent) | Team identity, rosters, casters and deletion rules |
| [Match Controller](Match-Controller) | Every match tab, from maps to config, plus the timer widget |
| [Tournament](Tournament) | Brackets, groups, playoffs and the board |
| [Assets](Assets) | Importing, organising and moving images |
| [Settings](Settings) | Port, server startup, LAN access, display options and storage |
| [Data and Backups](Data-and-Backups) | Where your data lives and how to move it |
| [Troubleshooting](Troubleshooting) | Common problems and fixes |
| [FAQ](FAQ) | Short answers to common questions |

## Where things are

- **Data folder:** `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\`
- **Default overlay URL:** `http://127.0.0.1:3001`
- **Logs:** the `logs\` folder inside the data folder

See [Data and Backups](Data-and-Backups) before deleting or moving anything.
