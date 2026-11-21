# Remote overlays

> [!IMPORTANT]
> This page describes remote access added in **post-v1 builds** on the main
> branch. The published **v1.0.0** release supports loopback and LAN access
> through **Allow LAN connections**, but it has no **Remote access** panel,
> Tailscale serving or Funnel. Funnel is not implemented in any build yet.

The overlay server normally listens on `127.0.0.1`. Remote access changes which
addresses can load the seven OBS Browser Sources; it does not change the overlay
HTML, the messages or the 1920×1080 Browser Source settings.

## Access methods

| Setting | URL shape | Availability |
|---|---|---|
| Both switches off | `http://127.0.0.1:<port>/overlays/<file>.html` | Loopback in v1.0.0 and post-v1 |
| **Local network** on | `http://<host-lan-ip>:<port>/overlays/<file>.html` | LAN in v1.0.0 via **Allow LAN connections**; a switch in post-v1 |
| **Tailscale** on | `https://<dnsname>/fgc/overlays/<file>.html` | Post-v1 builds |
| Both switches on | Both URL sets: `http://<host-lan-ip>:<port>/overlays/<file>.html` and `https://<dnsname>/fgc/overlays/<file>.html` | Post-v1 builds |
| Funnel | Not available | Not implemented; do not plan around it |

In post-v1 builds, use **Settings → Remote access**. **Local network** and
**Tailscale** are separate switches and both may be enabled at the same time.
When both are on, **Browser sources** lists a labelled set for each path once
Tailscale detection returns a DNS name: use the LAN set on the local network or
the Tailscale set on the tailnet. In v1.0.0 the same LAN behaviour is controlled
by **Allow LAN connections**; there is no Tailscale support.

## Before you start

Tailscale is **not installed or bundled** by the app. Use your own Tailscale
client, signed in and connected on the computer running the controller.

- The client must be connected. Settings shows the short CLI version and
  `Connected as <dnsname> (online)` when it is ready. The **Tailscale** switch
  stays disabled until then; **Refresh** re-reads the CLI.
- Your tailnet must have HTTPS certificates and MagicDNS available, and its ACLs
  must allow the OBS device to reach this machine on port 443. The app does not
  edit ACLs, manage certificates or install Tailscale.
- The app's listener still binds to loopback unless **Local network** is also
  enabled. Tailscale Serve forwards the tailnet paths to that local listener.
- There is **no password in the app**. With Tailscale enabled, your tailnet is
  the access boundary. With Local network enabled, anyone on the trusted network
  can read the overlays. Neither mode is public exposure.
- On Windows, Tailscale CLI calls run without a console window. A brief CMD
  popup is not expected; report the CLI version if one still appears.

## Share through Tailscale

1. In Settings, wait for the Tailscale client line to report **Connected as
   `<dnsname>` (online)**. Select **Refresh** if it was offline when the app
   opened.
2. Turn on the **Tailscale** switch, then start the overlay server (or leave
   **Start server on launch** on). Leave **Local network** on if you also need
   LAN access.
3. Confirm the status line says Tailscale Serve is active at `/fgc` and `/`.
4. On the **Overlay** page, select **Show URLs**; Settings shows the same list
   when URLs are revealed. Use `https://<dnsname>/fgc/overlays/<file>.html` for
   each Browser Source. There is no `?port=` suffix and no LAN IP in a
   Tailscale URL.
5. On another device signed in to the same tailnet, load one overlay and change
   a score in the controller. The overlay should update without reloading the
   Browser Source.

> [!NOTE]
> The app installs two Serve paths. `/fgc` carries the overlay pages and asset
> URLs. `/` carries the WebSocket connection that the bundled overlays open at
> the root of their page. Without the second path the page would load but never
> update.

> [!WARNING]
> The `/` handler makes the whole local overlay server reachable unprefixed on
> the tailnet, including `/`, `/health`, `/state`, `/overlays/...` and
> `/assets/...`. It is the same loopback service, not a second one, but it can
> take over a root Serve handler that another program had installed on this
> machine. Closing the app removes both app paths but does **not** restore the
> previous root handler. Check `tailscale serve status` before enabling
> Tailscale if the machine already serves something at `/`.

## LAN access is unchanged

For **Local network**, use **Allow LAN connections** in v1.0.0 or turn on
**Settings → Remote access → Local network** in later builds. Confirm the
Windows adapter is **Private** and that the inbound rule is **TCP**,
**Private** profile, limited to the **local subnet**. Use this PC's LAN IP in
place of `127.0.0.1`.

Tailscale itself does not need the app port opened through the Windows
firewall: the app still listens on loopback and Tailscale Serve forwards to it.
If **Local network** is also on, the LAN firewall rules still apply. Do not
open the port on a Public network or to any remote address.

## Stop sharing and clean up

- **Stop server** on the Overlay page removes the app's Serve handlers.
- Turning off the **Tailscale** switch removes them too, even when **Local
  network** stays on.
- Closing the app also removes them synchronously.

Teardown targets only the two paths the app owns. It does not use the port-wide
`off` or `tailscale serve reset`, so unrelated Serve handlers stay in place.
Check the result with:

```powershell
& "C:\Program Files\Tailscale\tailscale.exe" serve status
```

On a machine with no other Serve handlers the result is `No serve config`. If
other software uses Serve, its paths remain and only `/fgc` and `/` disappear.
A missing app path is treated as success, so stopping twice is safe.

## Troubleshooting

| Problem | What to do |
|---|---|
| The **Tailscale** switch is disabled | Start Tailscale and sign in. Settings only enables it when the CLI reports a connected client. Select **Refresh** after connecting. |
| Settings says `waiting for Tailscale address` | Detection has not returned this device's MagicDNS name yet. Wait a moment or select **Refresh**; the app will not invent a local URL when Tailscale is the only enabled path. |
| The URL still starts with `http://127.0.0.1` with Tailscale on | Detection has not supplied a DNS name. Refresh Tailscale and check that the client reports `Connected as <dnsname> (online)`. |
| Turning one switch off also turns the other off | That is not expected in post-v1 builds. Record the switch states and report it; **Local network** and **Tailscale** are independent. |
| The page loads but does not update | Check that `tailscale serve status` contains both `/fgc` and `/`. The root path is required for the overlays' WebSocket. |
| `tailscale serve status` shows a failed or missing path | Stop and start the server once. If it keeps failing, check whether another service owns port 443 or the root path, and whether your tailnet has HTTPS certificates enabled. |
| A device cannot open the HTTPS URL | Confirm it is signed in to the same tailnet, that MagicDNS resolves this machine, and that the tailnet ACL permits access to port 443. The app does not manage ACLs. |
| A pre-existing root Serve handler stopped working | The app takes over `/` while it serves and does not restore the previous configuration. Reconfigure that service after stopping the app. |
| A second PC cannot load a LAN URL | Use the LAN steps in [Overlays in OBS](Overlays-in-OBS) and confirm the Private firewall rule. Tailscale is a separate access path, not a fallback for a blocked LAN. |
| Public access is needed | Not available: Funnel is still pending. Do not expose the app port through a router or tunnel. |

## Related pages

- [Overlays in OBS](Overlays-in-OBS)
- [Settings](Settings)
- [Troubleshooting](Troubleshooting)
- [FAQ](FAQ)
