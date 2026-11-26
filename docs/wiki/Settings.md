# Settings

Settings controls the overlay network, startup, display options and asset
storage. Changes save automatically unless a field has its own **Apply**
button.

## Overlay network

| Setting | What it does |
|---|---|
| **Port** | The port the overlay server listens on. Default `3001`. Enter a value and select **Apply**. A running server restarts on the new port. |
| **Start server on launch** | Starts the overlay server automatically when the app opens. Off by default. |
| **Allow LAN connections** | In v1.0.0, binds on all interfaces so another machine can load overlays. Off by default. Post-v1 builds use **Remote access → Local network** instead. |
| **Asset URLs** | **Served URLs (/assets/…)** keeps messages small. **Inline data URLs** embeds every image in each message. |

> [!NOTE]
> Post-v1 builds replace **Allow LAN connections** with a **Remote access**
> panel containing two independent switches: **Local network** and **Tailscale**.
> Both may be on at the same time. **Browser sources** then shows a separate
> URL set for each enabled path, and secondary panels such as **Display**,
> **Asset storage** and **Overlay overrides** start collapsed. Tailscale is not
> available in the published v1.0.0 release; Funnel is not implemented. See
> [Remote overlays](Remote-Overlays).

> [!WARNING]
> Version 1 has no authentication. Turn on **Allow LAN connections** only on a
> trusted network, and allow the Windows Firewall prompt for private networks.

## Timer controls

Timer mode, alignment, leading zeroes, the target and the live clock are in the
timer widget above the status bar. Select its sliders icon to expand the
settings. The widget is available on every page; there is no separate Timer tab.

## Display

| Setting | What it does |
|---|---|
| **Use team colours** | Tints scoreboard elements with each team's primary colour. |

Tournament and watermark logos are configured in **Match Controller → Config**.

## Asset storage

- **Unused asset data** shows how many unreferenced user image files can be
  removed and their size.
- **Clean up now** removes image data that the saved and current projects no
  longer use.
- **Open overlays folder** opens the folder used for per-overlay HTML overrides.

Unreferenced image data is deleted automatically after a successful save and on
startup. See [Assets](Assets) and [Data and Backups](Data-and-Backups).

## Overlay overrides

You can replace one of the seven bundled overlay HTML files without rebuilding
the app. Copy a file with the same name into the **Overlays folder** shown here.

A missing, empty or unreadable override falls back to the embedded file. A
readable but malformed override is served as-is. The other overlays are
unaffected.

The default folder is:

```text
%APPDATA%\Fruiterra\Fruiterra Graphics Controller\overlays\
```

See [Overlays in OBS](Overlays-in-OBS).

## Local control

The **Local control** panel shows the loopback-only endpoint used by hardware
buttons and touch panels: `POST http://127.0.0.1:<port>/control`. The port shown
is the running server's port, or the configured port when the server is stopped.
The bearer token is a UUIDv4 string stored at
`%APPDATA%\Fruiterra\Fruiterra Graphics Controller\data\control-token`, beside
`app.json`; a `--data-dir` or `FGC_DATA_DIR` override moves it with the rest of
the project. It is hidden by default. **Reveal** shows it in the field, **Copy**
copies it without showing it on screen, and **Copy endpoint** copies the full
URL.

**Regenerate token** asks for confirmation because every Stream Deck or Touch
Portal client must be updated afterwards. Regenerating while the server is
running restarts it so the old token stops working immediately. The token is a
local secret: do not put it in a URL, screenshot, shared profile or support log.

Local network and Tailscale access change OBS Browser Source URLs only. They do
not make `/control` reachable from another machine. See
[Local control](Local-Control) for the request contract and client examples.

## Related pages

- [Overlays in OBS](Overlays-in-OBS)
- [Remote overlays (post-v1)](Remote-Overlays)
- [Local control](Local-Control)
- [Match Controller](Match-Controller)
- [Assets](Assets)
- [Data and Backups](Data-and-Backups)
