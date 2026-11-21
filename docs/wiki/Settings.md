# Settings

Settings controls the overlay network, startup, display options and asset
storage. Changes save automatically unless a field has its own **Apply**
button.

## Overlay network

| Setting | What it does |
|---|---|
| **Port** | The port the overlay server listens on. Default `3001`. Enter a value and select **Apply**. A running server restarts on the new port. |
| **Start server on launch** | Starts the overlay server automatically when the app opens. Off by default. |
| **Allow LAN connections** | In v1.0.0, binds on all interfaces so another machine can load overlays. Off by default. Post-v1 builds use **Remote access → Access scope → Local network** instead. |
| **Asset URLs** | **Served URLs (/assets/…)** keeps messages small. **Inline data URLs** embeds every image in each message. |

> [!NOTE]
> Post-v1 builds add **Remote access** with **This PC**, **Local network** and
> **Tailscale** scopes. Tailscale is not available in the published v1.0.0
> release. See [Remote overlays](Remote-Overlays).

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

## Related pages

- [Overlays in OBS](Overlays-in-OBS)
- [Match Controller](Match-Controller)
- [Assets](Assets)
- [Data and Backups](Data-and-Backups)
