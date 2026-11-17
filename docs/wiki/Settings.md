# Settings

Settings controls the overlay network, timer defaults, display options and asset
storage. Changes save automatically unless a field has its own **Apply** button.

## Overlay network

| Setting | What it does |
|---|---|
| **Port** | The port the overlay server listens on. Default `3001`. Enter a value and select **Apply**. A running server restarts on the new port. |
| **Allow LAN connections** | Binds on all interfaces so another machine can load overlays. Off by default. |
| **Asset URLs** | **Served URLs (/assets/…)** keeps messages small. **Inline data URLs** embeds every image in each message. |

> [!WARNING]
> Version 1 has no authentication. Turn on **Allow LAN connections** only on a
> trusted network, and allow the Windows Firewall prompt for private networks.

## Timer

These are display defaults for the timer payload:

- **Counting mode** — **Count up**, **Count down** or **Count to target**
- **Alignment** — **Left**, **Centre** or **Right**
- **Leading zeroes** — pad single-digit values with a zero
- **Standalone timer** — render the timer on its own instead of inside the
  scoreboard

The timer is control-only in version 1. There is no bundled timer overlay.

## Display

| Setting | What it does |
|---|---|
| **Use team colours** | Tint scoreboard elements with each team's primary colour. |
| **Watermark logo** | Shows the configured watermark, or **Bundled default (Peach)**. |
| **Tournament logo** | Shows the configured tournament logo, or **None**. |

Branding images can also be changed from **Match Controller → Config**.

## Asset storage

- **Unused asset data** shows how many files and how much space are unused.
- **Clean up now** removes unreferenced user images.
- **Open overlays folder** opens the override folder in Explorer.

Unreferenced image data is also cleaned after a successful save and on startup.

## Overlay overrides

The **Overlays folder** is where you can place a modified copy of any of the
seven overlay HTML files.

- Copy the file into the folder, keeping the same name.
- Edit the copy.
- Reload the Browser Source in OBS.

A missing, empty or unreadable override falls back to the embedded file. Only the
overridden overlay changes.

The default folder is:

```text
%APPDATA%\Fruiterra\Fruiterra Graphics Controller\overlays\
```

## Related pages

- [Overlays in OBS](Overlays-in-OBS)
- [Assets](Assets)
- [Data and Backups](Data-and-Backups)
