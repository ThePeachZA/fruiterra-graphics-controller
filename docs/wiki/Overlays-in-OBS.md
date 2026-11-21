# Overlays in OBS

The app serves seven plain HTML overlays. Each one is a separate OBS Browser
Source at **1920×1080** with a transparent background.

## The seven overlays

Replace `<base>` with the **Base URL** shown on the **Overlay** page. The
default is `http://127.0.0.1:3001`.

| Overlay | URL | What it shows |
|---|---|---|
| Game overlay | `<base>/overlays/game-overlay.html` | Score, team names, title, subtitle, current map, first-to and side indicators |
| Map select | `<base>/overlays/map-select.html` | The map picks and who chose each map |
| Hero bans | `<base>/overlays/hero-bans.html` | The banned hero for each team on each map |
| Map pool | `<base>/overlays/map-pool.html` | The map pool in the order set on the Map Pool tab |
| Home team roster | `<base>/overlays/team-roster-home.html` | The home team's players, roles, portraits and heroes |
| Away team roster | `<base>/overlays/team-roster-away.html` | The away team's players, roles, portraits and heroes |
| Casters row | `<base>/overlays/row-casters.html` | The assigned casters, panel and their social handles |

The **Overlay** page lists the same URLs with a **Copy URL** button for each.
URLs are hidden by default; select **Show URLs** to reveal them and **Hide
URLs** to hide them again.

## Add a Browser Source

1. In OBS, select **+** under **Sources** and choose **Browser**.
2. Give the source a name, for example `FGC – Game overlay`.
3. Set **Width** to `1920` and **Height** to `1080`.
4. Paste the overlay URL into **URL**.
5. Set **Custom frame rate** only if you have a reason to; the default is fine.
6. Turn **off** **Shutdown source when not visible**. The overlay must stay
   connected to keep its state.
7. If you want the entrance animations to replay every time the scene becomes
   active, turn on **Refresh browser when scene becomes active**. With the
   source kept loaded, OBS does not replay CSS animations on its own.
8. Leave the background transparent. Do not add a colour source behind the
   overlay unless your scene needs one.
9. Select **OK**, then check the source in the preview.

Repeat for each overlay you use. There is no fixed limit on the number of
Browser Sources; add one source per overlay file for each scene where it is
shown.

> [!TIP]
> Put each overlay in its own scene collection or scene group, then toggle the
> group rather than stopping and starting individual sources. Stopping a source
> disconnects it until OBS reloads it.

## Start and stop the server

- **Start server** starts the overlay server on the configured port.
- **Stop server** disconnects every overlay.
- **Restart** reconnects them. OBS sources reconnect within about two seconds
  and repaint from the current state.

The **Overlay** page shows the status, the base URL, the number of overlay files
and the listening port.

## Change the port

1. Open **Settings**.
2. Change **Port** and select **Apply**.
3. A running server restarts on the new port.
4. Update every OBS Browser Source URL to the new port.

The default is `3001`. Use a port between 1 and 65535.

## Show overlays on another machine

By default the server accepts connections from this PC only.

1. In v1.0.0, open **Settings** and turn on **Allow LAN connections**. In
   post-v1 builds, turn on **Settings → Remote access → Local network**.
2. Allow the Windows Firewall prompt for **private networks**. The show network
   must be set to **Private** in Windows, not **Public**.
3. On the other machine, replace `127.0.0.1` in the overlay URL with this PC's
   local IP address, for example `http://192.168.1.20:3001/overlays/game-overlay.html`.

If the other machine still cannot connect, add an inbound **TCP** rule for the
app on the **Private** profile, limited to the **local subnet**. Do not open the
port on public networks or to any remote address.

> [!WARNING]
> Version 1 has **no authentication**. Anyone who can reach the port can read
> the overlay state and load the overlays. Use LAN mode only on a network you
> trust, and turn it off when you are finished.

In post-v1 builds, **Settings → Remote access → Local network** does the same
job as **Allow LAN connections**. **Tailscale** is a separate switch in the
same panel; both switches can be on at once. Tailscale shares overlays through
your own client at `https://<dnsname>/fgc/overlays/<file>.html` and is **not**
in the published v1.0.0 release. See [Remote overlays](Remote-Overlays).

## Third-party and custom overlays

An overlay served from a different origin can pass the port explicitly:

```text
http://other-host/overlay.html?port=3001
```

Add `?host=<address>` when the app server is on another machine, for example:

```text
http://other-host/overlay.html?host=192.168.1.20&port=3001
```

A page served by the app uses its own origin automatically.

Third-party overlays receive the same messages as the bundled ones. This
includes `matchTimer` ticks and the start, pause and reset actions.

The timer is **control-only** in version 1. There is no bundled timer overlay;
use a third-party overlay if you need one on screen.

For the WebSocket contract, message types, payload fields and a copyable
listener, see [Custom overlays](Custom-Overlays).

## Replace one overlay without rebuilding

You can override a single overlay file:

1. Open **Settings → Overlay overrides** and note the **Overlays folder**, or
   select **Open overlays folder**.
2. Copy one of the seven HTML files into that folder, keeping the same name.
3. Edit the copy.
4. Reload the Browser Source in OBS.

A missing, empty or unreadable override falls back to the built-in file. The other
six overlays are unaffected.

The default folder is:

```text
%APPDATA%\Fruiterra\Fruiterra Graphics Controller\overlays\
```

## Related pages

- [Custom overlays](Custom-Overlays)
- [Settings](Settings)
- [Match Controller](Match-Controller)
- [Troubleshooting](Troubleshooting)
