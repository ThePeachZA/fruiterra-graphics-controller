# Troubleshooting

## The app

| Problem | What to do |
|---|---|
| Windows SmartScreen blocks the app | Expected for an unsigned build. Select **More info → Run anyway** after checking the SHA-256 hash against `SHA256SUMS.txt`. |
| The portable build closes immediately | WebView2 is probably missing. Install the Evergreen WebView2 runtime, or use the installer. |
| The app opens with an empty project | It created a new data folder. Check that `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\data\app.json` exists and that the folder was not renamed. |
| The project looks corrupted | Close the app, then check `data\app.json.bak`. See [Data and Backups](Data-and-Backups). |
| A second launch closes immediately | Only one instance runs. The second launch exits instead of starting a second window and server. |
| Text is clipped at 125% or 150% scaling | Try 100% scaling to confirm. If it only happens at higher scaling, report it with a screenshot and your Windows scaling value. |

## The overlay server

| Problem | What to do |
|---|---|
| **Start server** fails | Another process may be using the port. Change **Settings → Port**, select **Apply**, then update the OBS URLs. |
| The status is not **Running** | Start the server from the **Overlay** page. The Browser Source cannot connect while it is stopped. |
| An overlay is blank | Check that the server is running, that the URL matches the **Base URL**, and that the port in the URL matches **Settings → Port**. |
| An overlay stops updating | Confirm the Browser Source still points at the right URL. Restart the server once and let the source reconnect. |
| The overlay disconnects when you change scenes | In the Browser Source properties, turn off **Shutdown source when not visible**. |
| The entrance animation does not replay when a scene opens | Turn on **Refresh browser when scene becomes active**. With **Shutdown source when not visible** off, OBS keeps the page loaded and the animation only plays once. |
| An overlay reconnects in a loop | The server is stopped or restarting. Start it and reload the Browser Source once. |
| A second PC cannot load an overlay | Turn on **Allow LAN connections**, allow the Windows Firewall prompt for **private** networks, and use this PC's LAN IP instead of `127.0.0.1`. Both machines must be on the same network, and that network must be set to **Private** in Windows. If it is still blocked, add an inbound **TCP** rule for the app on the **Private** profile limited to the **local subnet**. Do not open the port to public networks. |
| The overlay shows old images | Content-addressed URLs change when the image changes. Reload the Browser Source once if OBS cached the old frame. |

## Data and editing

| Problem | What to do |
|---|---|
| A team cannot be deleted | It is used by a tournament or the current match. The dialog lists every reference. Clear them, or select **Remove references and delete**. |
| A completed bracket match is rejected | A completed match needs a winner. Set the scores or choose a **Winner** in the match dialog. |
| A team disappeared from a group | Removing a team from a group does not delete the team. Add it back from the group's team picker. |
| Tournament data looks wrong after a crash | The app repairs malformed tournament data on load. Check the match dialog and the **Settings** tab, then re-enter any result that was reset. |
| An image import is rejected | The file must be PNG, JPEG, WebP, GIF or SVG and no larger than 20 MB. SVG files containing scripts or external references are rejected. |
| A URL import fails | Check the address points directly at an image, that the host is reachable, and that the image is under 20 MB. |
| An overlay override is ignored | The file must be in the **Overlays folder** with exactly the same filename as the bundled overlay. Reload the Browser Source. A missing, empty or unreadable override falls back to the built-in file. |

## Timer and fonts

| Problem | What to do |
|---|---|
| There is no timer on screen | The timer is control-only in version 1. There is no bundled timer overlay. Third-party overlays receive the timer messages. |
| The timer runs but nothing changes in OBS | Check that your third-party overlay subscribes to `matchTimer`. The bundled overlays do not display it. |
| Overlay text looks different from older graphics | Version 1 uses Archivo behind the `FGC Display*` family names. The layout is unchanged; the typeface is not Apotek. |

## Getting help

When reporting a problem, include:

- The app version.
- Windows version and display scaling.
- OBS version and whether the source is a Browser Source.
- The exact steps that reproduce the problem.
- The newest file from the `logs\` folder.
- A screenshot of the app and the OBS preview, if possible.

Open an issue at
[github.com/ThePeachZA/fruiterra-graphics-controller/issues](https://github.com/ThePeachZA/fruiterra-graphics-controller/issues).
