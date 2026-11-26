# Remote browser control — design pass

Status: **draft for owner review**. IDE-002 was started on 2026-11-26; nothing in
this document is implemented yet. Approval is requested on the decisions in
§11 before any code is written.

This document answers the two open questions recorded in IDE-002:

1. Which actions are safe to expose first?
2. What credential or token model is acceptable for LAN and tailnet use?

It deliberately extends the control API shipped in IMP-003 instead of inventing
a second one. The authenticated local endpoint, its action enum and its replay
guard stay the shared foundation.

---

## 1. Goal and non-goals

The controller UI is desktop-only. The owner wants to open a small page on
another device on the local network or the tailnet and press the same Match
Controller actions available on the desktop app. The page must follow the
existing **Local network** and **Tailscale** switches and the same URL reveal
rules as the OBS Browser Sources.

Non-goals for this design:

- **No Funnel or public exposure.** D8-5 remains a separate, unapproved item.
  Nothing here makes the app reachable from the internet.
- **No v1 changes.** The payloads, schema, protocol, auto-overlay messages and
  the seven frozen overlay HTML files are untouched. This is server routing and
  a standalone page, not a new overlay.
- **No new visual system.** The page uses the app's existing design tokens and
  components. No new fonts, assets or CDN dependencies.
- **No replacement for the desktop controller.** The page exposes a bounded set
  of live actions, not every edit the desktop UI can make.
- **No rollout before HMAC and QA.** LAN or Tailscale control is not enabled in
  a release until HMAC signing and a real second-device QA case (A94) pass.

---

## 2. Decisions at a glance

| Question | Decision |
|---|---|
| One API or two? | **One.** `POST /control` and the `ControlAction` enum are extended, not duplicated. |
| Credential | The existing control token is the single secret. Loopback clients use bearer; remote clients sign with HMAC-SHA256. |
| How the page gets the secret | The operator types the token into the page. It stays in JavaScript memory only — never in a URL, `localStorage`, or a request body. |
| When remote control works | Only when **Local network** or **Tailscale** is on. With both off the page is not served and remote auth is refused. |
| First action set | The six show actions already shipped in IMP-003. No match-data writes in the first rollout. |
| Later action sets | Bounded writes first (score ±, timer, advance map); destructive edits deferred behind a confirmation and undo design. |
| URL shapes | The same pattern as Browser Source URLs: `http://<host-lan-ip>:<port>/remote` and `https://<dnsname>/fgc/remote`. |

---

## 3. What stays fixed

These are contracts, not choices:

- `POST /control` keeps its current JSON body for loopback callers: `action`,
  `timestamp`, `nonce`, and `scene` for `set_scene`. Stream Deck and Touch Portal
  examples in `examples/control/` keep working unchanged.
- The replay rules stay as shipped: timestamp skew at most 60 seconds, nonce
  retained for five minutes, 30 requests per minute per peer, `no-store` on
  control responses, and no permissive CORS.
- The token file stays at `<app data root>\data\control-token` and regenerating
  the token still revokes every client at once.
- The Remote access switches keep their current meaning: they decide which
  addresses can reach the app. They do not gain a new "control" toggle.
- Tailscale Serve keeps its two paths, `/fgc` for prefixed overlay traffic and
  `/` for the overlays' WebSocket connection.
- No overlay file, wire message type, or persisted schema is added or changed.

---

## 4. Architecture

```text
Hardware buttons (loopback)          Browser on another device (LAN / tailnet)
  bearer token                         operator types token once; page signs with HMAC
        |                                             |
        +------------> POST /control <----------------+
                            |
                  auth mode chosen by peer + switch state
                            |
                  replay / rate-limit guards (shared)
                            |
                  ControlAction -> UI task -> existing app primitives
                            |
                  existing overlay connection repaints OBS Browser Sources
```

There is one endpoint and one action channel. What changes is the **auth mode**
and the **server-side gating** around the same handler:

- **Loopback bearer (shipped).** A local peer sends `Authorization: Bearer
  <token>`. This is what the Stream Deck and Touch Portal examples do.
- **Remote HMAC (new).** A non-loopback peer, or a request that arrives through
  the Tailscale proxy, sends `Authorization: HMAC-SHA256 <signature>` instead.
  The secret is never transmitted.

The browser page is a small static HTML document served by the same overlay
server. It is deliberately not a Dioxus view inside the desktop app: it has to
work on a phone without shipping the app UI to the browser.

---

## 5. Security model

### 5.1 One secret, two auth modes

The control token (a UUIDv4) remains the only credential. The app already
shows, copies and regenerates it in **Settings → Local control**.

- **Bearer mode** is accepted from loopback peers exactly as today. The server
  stores a SHA-256 digest and compares in constant time.
- **HMAC mode** uses the same token bytes as the signing key. The server must
  hold the raw token in memory for the lifetime of the bridge. That adds no new
  exposure: the app already reads the raw token to display it in Settings. The
  in-memory value must live in a redacted newtype that is never logged,
  serialised, or included in an error message.

Using one secret keeps the operator's mental model simple: **the token is the
write credential**. Anyone who has it can drive show actions; regenerating it
revokes every client, local and remote.

### 5.2 Canonical request and signature

Remote clients sign the exact bytes they send. For a POST to `/control`:

```text
FGC-CONTROL-V1
<timestamp>
<nonce>
POST
/control
<lowercase-hex SHA-256 of the exact JSON body bytes>
```

Each line ends with `\n`. The signature is lowercase hex
`HMAC-SHA256(key = <token bytes>, message = <the lines above>)`, sent as:

```http
Authorization: HMAC-SHA256 <signature>
```

The body already contains `timestamp` and `nonce`; repeating them in the signed
message makes the replay checks explicit and stops a client from silently
changing either value after signing. The server reconstructs the message from
the raw request bytes, the parsed timestamp and nonce, the method and the path,
then compares the signature in constant time. A malformed or missing signature
is rejected before the action is parsed further.

`FGC-CONTROL-V1` is a domain separator so a signature from another protocol
cannot be replayed here. A future key-rotation scheme can add a key id without
changing the message shape.

### 5.3 Switch gating

Define `remote_control_enabled = local_network_on || tailscale_on`.

| State | Page | Remote auth | Loopback bearer |
|---|---|---|---|
| Both switches off | Not served (404) | Refused | Works as shipped |
| Local network on | `http://<host-lan-ip>:<port>/remote` | HMAC accepted | Works |
| Tailscale on | `https://<dnsname>/fgc/remote` | HMAC accepted | Works |
| Both on | Both URLs, revealed together | HMAC accepted | Works |

The switches already control the network boundary: with both off the listener
is loopback-only and Tailscale Serve paths are removed, so there is no route to
the page at all. The explicit `remote_control_enabled` check is defence in depth
and gives a clear status line in Settings.

Requests that arrive through Tailscale Serve appear to come from loopback
because Serve connects to `127.0.0.1`. Peer IP alone therefore cannot distinguish
a tailnet client from a local one. HMAC is what makes that distinction safe: a
tailnet client must possess the token and sign; a local hardware client can keep
using bearer. During implementation, confirm whether Serve adds a forwarding
header such as `X-Forwarded-For`; if it does, treat such requests as remote for
logging and rate-limit purposes, but do not rely on the header for authorisation.

### 5.4 Replay and rate limits

The existing guards are reused for both auth modes:

- Timestamp within 60 seconds of the server clock.
- Nonce 8–128 characters, alphanumeric plus `-` and `_`.
- Nonce retained for five minutes. The cache key becomes `(peer, nonce)` so two
  devices using the same random value do not reject each other. Tailnet clients
  share the proxy's loopback address, so the page generates nonces with
  `crypto.randomUUID()`; collisions are negligible.
- 30 requests per minute per observed peer. Tailnet clients share one bucket;
  that is acceptable for a single-operator tool and is documented as a known
  limitation rather than silently raised.
- Every control response, success or error, keeps `Cache-Control: no-store`.

### 5.5 Why bearer is not acceptable over the network

A bearer token is transmitted with every request. On plain LAN HTTP a passive
observer could capture it and gain write access; on Tailscale HTTPS the transport
is protected but the token would still be present in browser history tooling and
proxy logs. HMAC keeps the secret on the device where the operator typed it and
sends only a single-use signature. The replay window is short and the signature
cannot be reused after the nonce expires.

### 5.6 Tailscale root handler

The app's root Serve path proxies the whole local listener, so `/control` is
also reachable at `https://<dnsname>/control`. That does not grant extra
privilege: the caller still needs the token, which is also the HMAC key. The
design does not attempt to hide the endpoint behind the proxy. The browser page
uses the prefixed `/fgc/remote` path for consistency with overlays, and
implementation may narrow the root Serve target to a WebSocket-only mount if
testing shows it is safe to do so without breaking A44.

---

## 6. Action set and phasing

The open question "which actions are safe to expose first" is answered by
phasing. Every action is a variant of the existing `ControlAction` enum and is
applied through `AppStore::apply_control_action`, so phase changes do not add a
new endpoint or a new wire protocol.

### Phase 1 — show control (ship with the page)

| Action | Why it is safe |
|---|---|
| `show` / `hide` | Mirrors the desktop show toggle. A wrong press changes visibility, not the match record. |
| `force_update` | Resends current state; no state change. |
| `set_scene` / `next_scene` / `previous_scene` | Session-only auto-overlay selection; already QA'd in A52 and A93. |

These are the six actions already documented in
[`docs/wiki/Local-Control.md`](wiki/Local-Control.md). No new action names are
needed for the first rollout.

### Phase 2 — bounded writes (after owner review)

| Action | Shape | Guard |
|---|---|---|
| `score_adjust` | `team: "a" \| "b"`, `delta: 1 \| -1` | Clamped to the score domain; one press, one point. |
| `timer` | `command: "start" \| "pause" \| "stop"` | Reuses the existing timer methods; idempotent where the desktop UI is. |
| `advance_map` | no payload | Selects the next enabled map card only; does not edit picks or bans. |

These are high-value, small writes with an obvious manual correction path on
the desktop controller. They are not enabled until the owner has reviewed a
prototype.

### Phase 3 — deferred, needs its own design

Exact score set, map and ban edits, swap sides, reset maps, and new/load match
are multi-field or destructive. Exposing them needs a confirmation step and an
undo story on the page, plus a dedicated QA pass. They are out of scope here.

---

## 7. Browser page

- Served as a static HTML document at `/remote` (LAN) and `/fgc/remote`
  (Tailscale). The same file and JavaScript work for both: a relative POST to
  `control` resolves to the internal `/control` on LAN and after the `/fgc`
  prefix is stripped by Serve.
- **Inert until the operator enters the token.** Loading the page grants
  nothing. The token is kept in a JavaScript variable for the lifetime of the
  tab; it is cleared on reload and never written to a URL, cookie or storage.
- Buttons are rendered only for the enabled action phase. A status area shows
  the current scene, the last accepted action, and the server's error codes
  (`unauthorised`, `stale_timestamp`, `replayed_nonce`, `rate_limited`,
  `control_unavailable`).
- States that must be designed: server stopped (buttons disabled with an
  explanation), token not entered, wrong token, stale request, replayed nonce,
  rate limited, and a successful press.
- The page is reachable at phone width, works with a keyboard, labels every
  control, announces status to a screen reader, and respects reduced motion.
- Visual sign-off from the owner is required before implementation, in line
  with the project's rule for user-facing surface work.

---

## 8. Settings and URL reveal

No new switch is added. The **Local control** panel gains a short subsection:

- An availability line: **Off** when both remote switches are off, otherwise
  **Local network**, **Tailscale**, or **Local network + Tailscale**.
- The remote page URLs appear in the same reveal/hide group as the OBS Browser
  Source URLs. They follow the existing **Show URLs** control and the `reveal_urls`
  state, so they are hidden by default and shown together with the overlay URLs.
- A note that remote requests use HMAC and that the bearer token is the secret
  to type into the page. Regenerating the token invalidates all remote sessions;
  an open page loses its in-memory secret on reload.

---

## 9. Verification plan

Automated checks in `crates/server/src/control.rs`:

- HMAC accepts a correctly signed canonical request and rejects a tampered
  body, timestamp, nonce, path or method.
- Bearer is accepted from a loopback peer and rejected from a non-loopback peer.
- HMAC is refused when `remote_control_enabled` is false.
- A replayed `(peer, nonce)` pair is rejected with `replayed_nonce`.
- Signature comparison is constant time.

Manual QA: add **A94 — Remote browser control from a second device** to
`docs/manual-qa-runbook.md` §7, alongside A42 (LAN) and A44 (Tailscale). Pass
conditions:

- With both switches off, neither remote URL resolves and the page is not
  served.
- With Local network on, the second PC loads the LAN page, the operator types
  the token, and `show`/`hide` update the overlays without a reload.
- With Tailscale on, a second tailnet device loads the HTTPS page and does the
  same.
- Both on: both URLs are revealed and both work.
- Wrong token, stale timestamp, replayed nonce and rate limit return the
  documented codes and do not change overlay state.
- Regenerating the token stops the open page from signing further requests.
- Stopping the server and turning either switch off clean up the page and Serve
  paths without disturbing unrelated handlers (reuse the A46 teardown check).

---

## 10. Implementation order

1. **HMAC foundation.** Add signing and verification to the server with unit
   tests. No behaviour change for loopback bearer clients.
2. **Gating and page.** Add `remote_control_enabled`, serve `/remote`, wire the
   HMAC header into the existing handler, and add the Settings status/URLs.
3. **LAN QA first.** Run A94 on the LAN path; this is the simpler transport and
   catches request-shape bugs before HTTPS is involved.
4. **Tailscale path.** Confirm the `/fgc/remote` route and the root-proxy note
   in §5.6 on a second tailnet device; complete the Tailscale half of A94.
5. **Docs.** Update `docs/wiki/Local-Control.md`, `docs/wiki/Remote-Overlays.md`
   and the examples runbook only after the QA pass.
6. **Phase 2 actions.** Separate slice after owner review; Phase 3 remains a
   separate design.

D8-5 (Funnel) is not a step in this plan. It needs its own design and approval.

---

## 11. Owner approval checklist

Please confirm or correct:

- [ ] One secret and one API; bearer stays local, HMAC used for remote.
- [ ] The browser page is inactive until the token is typed, and the token is
      never stored in the browser.
- [ ] Phase 1 exposes only the six existing show actions; score, timer and map
      writes come later.
- [ ] Remote URLs follow the Browser Source reveal rules at `/remote` and
      `/fgc/remote`.
- [ ] No Funnel, no v1 changes, and A94 must pass before a release enables this.
- [ ] The Phase 2 write set in §6 is the right order of priority.

---

## 12. Addendum — match-controller actions and destructive edits

Owner direction after Phase 1: remote control must also drive elements of the
Match Controller, including **map/ban edits**. This addendum replaces the
Phase 1-only action list in §6 for the next slice. It is **draft for owner
review**; implementation waits for approval of this section.

### 12.1 Action set

All new actions remain variants of the existing `ControlAction` enum on
`POST /control`. They use the same bearer/HMAC authentication, replay guard and
UI-task dispatch as Phase 1.

| Action | Fields | Effect on the existing app |
|---|---|---|
| `score_adjust` | `team: "a" \| "b"`, `delta: 1 \| -1` | Reads the current score, clamps at zero, switches to manual scoring and applies the new value. |
| `timer` | `command: "start" \| "pause" \| "stop"` | Calls the existing timer controller; stop resets to the mode's initial seconds. |
| `set_current_map` | `index: usize` | Marks one map slot current; out-of-range values are ignored. |
| `set_map_pick` | `index: usize`, `team: "a" \| "b" \| "none"` | Sets the slot's picking team. Refused when `loser_picks` is on. |
| `set_map_result` | `index: usize`, `outcome: "a" \| "b" \| "draw" \| "unplayed"` | Sets both map scores (1–0, 0–1, 0–0) and the completed flag; unplayed clears completion. |
| `set_hero_ban` | `index: usize`, `team: "a" \| "b"`, `hero: string` | Sets one hero ban; an empty or whitespace hero clears the ban. |
| `set_first_ban_team` | `index: usize`, `team: "a" \| "b" \| "none"` | Sets which team bans first on that slot. |

No `reset_maps`, `reset_hero_bans`, match replacement, team swaps or new-match
actions are exposed remotely in this slice. They are destructive beyond a
single-field undo and stay desktop-only.

### 12.2 Confirmation

`score_adjust`, `timer`, and `set_current_map` are single-step, like Phase 1.
Pick, result, first-ban-team and hero-ban edits are **two-step**:

1. The first press prepares the action and shows a confirmation panel naming
   the slot, field and new value. Nothing is sent.
2. **Confirm** sends the signed request. **Cancel** discards it.
3. The request carries `confirm: true`; a prepared action sent without it is
   rejected with `confirmation_required`. The field is part of the signed body,
   so it cannot be added by a proxy.

This gives the operator a chance to check the slot number on a phone screen
before a match record changes.

### 12.3 Undo

Every accepted mutating request returns an `undo` token derived from the
request nonce. For the next five minutes, `POST /control` with
`action: "undo"`, `undo: <token>`, plus a fresh timestamp and nonce restores
the previous value of that one field. The app stores the inverse in memory
only; it is not persisted and is lost on restart. Undo is single-use and
subject to the same replay and rate-limit guards. A stale or already-used undo
token returns `undo_unavailable`. The browser page offers an **Undo** button
in the success status for the duration of the window.

Undo is deliberately field-scoped. It does not restore deleted matches, do
cascading repairs, or reverse a timer that has kept running. The confirmation
panel states that plainly.

### 12.4 Page and status

The remote page gains a compact **Match** section below Show actions, grouped
as Score, Timer, Maps and Bans. It disables all match controls when there is
no current match, when `loser_picks` blocks manual picks, or when the server
is unreachable. Successful actions announce the applied value and expose the
Undo button; rejected actions show the server's error code in plain language
and re-enable the controls. Long hero names wrap and the confirm panel stays
inside the viewport at phone width. Keyboard focus moves into the confirm
panel on prepare and back to the originating button on cancel/confirm.

### 12.5 Verification

- Server unit tests for each new wire shape, the `confirmation_required`
  refusal, undo inverse restoration, single-use undo and invalid indices.
- App tests for score clamping, loser-pick refusal, ban clearing and no-current
  match no-ops, using the existing domain helpers.
- Second-device QA case A94 is extended with one score adjustment, one timer
  press, one map pick/result, one hero ban, one confirm/cancel pair and one
  undo, on both LAN and Tailscale.

### 12.6 Approval checklist

- [ ] Expose the seven match actions in §12.1 and keep reset/new-match actions
      desktop-only.
- [ ] Two-step confirmation for pick, result, first-ban-team and hero-ban edits.
- [ ] Five-minute in-memory, single-use undo for every accepted write.
- [ ] Disabling rules for no match, loser-picks and unreachable server.
