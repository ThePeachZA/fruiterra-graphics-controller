//! Standalone browser control page.
//!
//! This page is served only while a remote control path is enabled. It is
//! intentionally not a Dioxus view: it has to work on a phone without shipping
//! the desktop UI to the browser. The page uses the same colour and type tokens
//! as the app UI, inlined as CSS because it is a separate document.
//!
//! The operator types the control token into the page. It is kept in a
//! JavaScript variable for the life of the tab and is never written to a URL,
//! storage, or a request body. Signing uses a small SHA-256/HMAC implementation
//! rather than Web Crypto: `crypto.subtle` is undefined on the plain HTTP LAN
//! origin, while `crypto.getRandomValues` is available there.

/// Returns the remote control HTML document.
#[must_use]
pub(crate) fn html() -> &'static str {
    HTML
}

const HTML: &str = r####"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="dark">
<title>Fruiterra remote control</title>
<style>
  :root {
    color-scheme: dark;
    --bg: #0b0d12;
    --panel: #151922;
    --line: #2a3140;
    --text: #eef2f8;
    --muted: #8d99ad;
    --accent: #ff7a1a;
    --accent-soft: rgb(255 122 26 / 14%);
    --success: #3ecf8e;
    --danger: #ef5b5b;
    --radius: 0.7rem;
    --shadow: 0 12px 32px rgb(0 0 0 / 28%);
    --font-ui: "Inter", "Segoe UI", system-ui, sans-serif;
    --font-mono: "Cascadia Mono", "Consolas", monospace;
  }
  * { box-sizing: border-box; }
  html, body { margin: 0; min-height: 100%; }
  body {
    background: var(--bg);
    color: var(--text);
    font: 400 1rem/1.5 var(--font-ui);
    -webkit-font-smoothing: antialiased;
  }
  .shell {
    width: min(100%, 52rem);
    margin: 0 auto;
    padding: clamp(1rem, 4vw, 2.5rem);
    display: grid;
    gap: 1rem;
  }
  .head { display: grid; gap: 0.35rem; }
  .eyebrow {
    margin: 0;
    color: var(--accent);
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }
  h1 { margin: 0; font-size: clamp(1.6rem, 5vw, 2.4rem); line-height: 1.1; }
  h2 { margin: 0; font-size: 1.05rem; }
  .lede, .hint, .foot { margin: 0; color: var(--muted); font-size: 0.875rem; }
  .card {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    padding: clamp(1rem, 3vw, 1.5rem);
    display: grid;
    gap: 1rem;
    min-width: 0;
  }
  .unlock, .scene-form { display: grid; gap: 0.5rem; min-width: 0; }
  label { font-size: 0.875rem; font-weight: 600; }
  input, select {
    width: 100%;
    min-width: 0;
    min-height: 44px;
    padding: 0.55rem 0.75rem;
    color: var(--text);
    background: var(--bg);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    font: inherit;
  }
  input { font-family: var(--font-mono); font-size: 0.85rem; }
  button {
    min-height: 44px;
    padding: 0.6rem 1rem;
    color: var(--text);
    background: var(--accent-soft);
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    font: inherit;
    font-weight: 650;
    cursor: pointer;
    transition: background-color 120ms ease, border-color 120ms ease;
  }
  button:hover:not(:disabled) { background: var(--accent); color: #140b04; }
  button:disabled { opacity: 0.45; cursor: not-allowed; }
  button.ghost { background: transparent; border-color: var(--line); color: var(--muted); }
  button.ghost:hover:not(:disabled) { background: var(--panel); color: var(--text); }
  :focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
    gap: 0.75rem;
    min-width: 0;
  }
  .grid button { width: 100%; }
  .status, .alert, .ok {
    margin: 0;
    padding: 0.7rem 0.9rem;
    border-radius: var(--radius);
    font-size: 0.875rem;
    overflow-wrap: anywhere;
  }
  .status { background: var(--panel); border: 1px solid var(--line); color: var(--muted); }
  .status[data-tone="success"] { border-color: var(--success); color: var(--success); }
  .alert { background: rgb(239 91 91 / 12%); border: 1px solid var(--danger); color: #ffc7c7; }
  .ok { background: rgb(62 207 142 / 10%); border: 1px solid var(--success); color: var(--success); }
  .session { display: flex; flex-wrap: wrap; gap: 0.75rem; align-items: center; min-width: 0; }
  .session .ok { flex: 1 1 14rem; }
  .scene-form { grid-template-columns: 1fr; }
  .foot { padding-bottom: 1rem; overflow-wrap: anywhere; }
  [hidden] { display: none !important; }
  @media (min-width: 38rem) {
    .scene-form { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto; align-items: end; }
  }
  @media (prefers-reduced-motion: reduce) {
    * { transition: none !important; }
  }
</style>
</head>
<body>
<main class="shell">
  <header class="head">
    <p class="eyebrow">Fruiterra Graphics Controller</p>
    <h1>Remote control</h1>
    <p class="lede">Press show actions from this device. Enter the local control token to unlock the buttons.</p>
  </header>

  <section class="card" aria-labelledby="access-title">
    <h2 id="access-title">Access</h2>
    <form id="unlock-form" class="unlock">
      <label for="token">Control token</label>
      <input id="token" name="token" type="password" autocomplete="off" spellcheck="false"
             aria-describedby="token-help" required>
      <p class="hint" id="token-help">From Settings → Local control. It stays in this tab only.</p>
      <button type="submit" id="unlock">Unlock controls</button>
    </form>
    <div class="session" id="session" hidden>
      <p class="ok" role="status">Token active. Controls unlocked.</p>
      <button type="button" class="ghost" id="forget">Forget token</button>
    </div>
  </section>

  <p class="status" id="status" role="status" aria-live="polite" aria-busy="true">Checking connection…</p>
  <p class="alert" id="alert" role="alert" hidden></p>

  <section class="card" aria-labelledby="actions-title">
    <h2 id="actions-title">Show actions</h2>
    <div class="grid" id="actions" aria-busy="false">
      <button type="button" data-action="show">Show overlays</button>
      <button type="button" data-action="hide">Hide overlays</button>
      <button type="button" data-action="force_update">Force update</button>
      <button type="button" data-action="next_scene">Next scene</button>
      <button type="button" data-action="previous_scene">Previous scene</button>
    </div>
    <form class="scene-form" id="scene-form">
      <div>
        <label for="scene">Scene</label>
        <select id="scene" name="scene">
          <option value="game-overlay.html">Game overlay</option>
          <option value="map-select.html">Map select</option>
          <option value="hero-bans.html">Hero bans</option>
          <option value="map-pool.html">Map pool</option>
          <option value="team-roster-home.html">Team roster — home</option>
          <option value="team-roster-away.html">Team roster — away</option>
          <option value="row-casters.html">Casters row</option>
          <option value="row-hosts.html">Hosts row</option>
          <option value="row-interviews.html">Interviews row</option>
        </select>
      </div>
      <button type="submit" data-action="set_scene">Set scene</button>
    </form>
  </section>

  <p class="foot">This page is reachable only on an enabled remote path. The token is never stored in the browser or sent in a request.</p>
</main>

<script>
(() => {
  "use strict";

  const K = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
  ];
  const INITIAL = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
  ];
  const encoder = new TextEncoder();
  const ror = (x, n) => (x >>> n) | (x << (32 - n));

  function sha256(bytes) {
    const h = INITIAL.slice();
    const bitLength = bytes.length * 8;
    const paddedLength = bytes.length + 1 + ((56 - ((bytes.length + 1) % 64) + 64) % 64) + 8;
    const message = new Uint8Array(paddedLength);
    message.set(bytes);
    message[bytes.length] = 0x80;
    const view = new DataView(message.buffer);
    view.setUint32(paddedLength - 8, Math.floor(bitLength / 0x100000000), false);
    view.setUint32(paddedLength - 4, bitLength >>> 0, false);

    const w = new Uint32Array(64);
    for (let offset = 0; offset < paddedLength; offset += 64) {
      for (let i = 0; i < 16; i++) w[i] = view.getUint32(offset + i * 4, false);
      for (let i = 16; i < 64; i++) {
        const x = w[i - 15];
        const y = w[i - 2];
        const s0 = ror(x, 7) ^ ror(x, 18) ^ (x >>> 3);
        const s1 = ror(y, 17) ^ ror(y, 19) ^ (y >>> 10);
        w[i] = (w[i - 16] + s0 + w[i - 7] + s1) >>> 0;
      }
      let [a, b, c, d, e, f, g, old] = h;
      for (let i = 0; i < 64; i++) {
        const big1 = ror(e, 6) ^ ror(e, 11) ^ ror(e, 25);
        const choose = (e & f) ^ (~e & g);
        const t1 = (old + big1 + choose + K[i] + w[i]) >>> 0;
        const big0 = ror(a, 2) ^ ror(a, 13) ^ ror(a, 22);
        const majority = (a & b) ^ (a & c) ^ (b & c);
        const t2 = (big0 + majority) >>> 0;
        old = g; g = f; f = e; e = (d + t1) >>> 0;
        d = c; c = b; b = a; a = (t1 + t2) >>> 0;
      }
      h[0] = (h[0] + a) >>> 0; h[1] = (h[1] + b) >>> 0;
      h[2] = (h[2] + c) >>> 0; h[3] = (h[3] + d) >>> 0;
      h[4] = (h[4] + e) >>> 0; h[5] = (h[5] + f) >>> 0;
      h[6] = (h[6] + g) >>> 0; h[7] = (h[7] + old) >>> 0;
    }
    const out = new Uint8Array(32);
    const outView = new DataView(out.buffer);
    for (let i = 0; i < 8; i++) outView.setUint32(i * 4, h[i], false);
    return out;
  }

  function hmacSha256(key, message) {
    let block = key;
    if (block.length > 64) block = sha256(block);
    const innerKey = new Uint8Array(64);
    innerKey.set(block);
    const ipad = new Uint8Array(64);
    const opad = new Uint8Array(64);
    for (let i = 0; i < 64; i++) {
      ipad[i] = innerKey[i] ^ 0x36;
      opad[i] = innerKey[i] ^ 0x5c;
    }
    const inner = new Uint8Array(64 + message.length);
    inner.set(ipad);
    inner.set(message, 64);
    const innerHash = sha256(inner);
    const outer = new Uint8Array(96);
    outer.set(opad);
    outer.set(innerHash, 64);
    return sha256(outer);
  }

  const hex = (bytes) => Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
  const randomNonce = () => {
    const bytes = new Uint8Array(16);
    crypto.getRandomValues(bytes);
    return "n" + hex(bytes);
  };

  const status = document.getElementById("status");
  const alertBox = document.getElementById("alert");
  const unlockForm = document.getElementById("unlock-form");
  const sceneForm = document.getElementById("scene-form");
  const tokenInput = document.getElementById("token");
  const session = document.getElementById("session");
  const forget = document.getElementById("forget");
  const actions = document.getElementById("actions");
  const sceneSelect = document.getElementById("scene");

  let token = null;
  let online = false;
  let pending = false;

  const ERROR_TEXT = {
    unauthorised: "The token was not accepted. Check it in Settings → Local control.",
    loopback_only: "Remote control is not enabled on the controller.",
    stale_timestamp: "The request was too old. Check the controller's clock and try again.",
    invalid_nonce: "The request could not be identified. Try the button again.",
    replayed_nonce: "That press was already used. Try the button again.",
    rate_limited: "Too many presses at once. Wait a moment and try again.",
    control_unavailable: "The controller is not ready. Start the overlay server and try again.",
    invalid_request: "The controller could not read that request. Try the button again."
  };

  function say(message, tone) {
    status.textContent = message;
    status.setAttribute("aria-busy", "false");
    if (tone === "success") status.dataset.tone = "success";
    else delete status.dataset.tone;
  }

  function warn(message) {
    alertBox.textContent = message;
    alertBox.hidden = false;
  }

  function clearAlert() {
    alertBox.textContent = "";
    alertBox.hidden = true;
  }

  function syncButtons() {
    const disabled = !token || !online || pending;
    for (const button of actions.querySelectorAll("button")) button.disabled = disabled;
    for (const button of sceneForm.querySelectorAll("button")) button.disabled = disabled;
    actions.setAttribute("aria-busy", pending ? "true" : "false");
    const reason = !online
      ? "Cannot reach the controller."
      : !token
        ? "Enter the control token to unlock the buttons."
        : "Ready.";
    if (!pending) say(reason);
  }

  async function checkConnection() {
    say("Checking connection…");
    status.setAttribute("aria-busy", "true");
    try {
      const response = await fetch("health", { cache: "no-store" });
      online = response.ok;
    } catch {
      online = false;
    }
    syncButtons();
    if (!online) warn("Cannot reach the controller. Check that the overlay server is running.");
    else clearAlert();
  }

  async function press(action) {
    if (!token || !online || pending) return;
    pending = true;
    clearAlert();
    say("Sending " + action.replace(/_/g, " ") + "…");
    syncButtons();

    const body = { action };
    if (action === "set_scene") body.scene = sceneSelect.value;
    body.timestamp = Math.floor(Date.now() / 1000);
    body.nonce = randomNonce();
    const text = JSON.stringify(body);
    const bodyHash = hex(sha256(encoder.encode(text)));
    const message =
      "FGC-CONTROL-V1\n" + body.timestamp + "\n" + body.nonce +
      "\nPOST\n/control\n" + bodyHash + "\n";
    const signature = hex(hmacSha256(encoder.encode(token), encoder.encode(message)));

    try {
      const response = await fetch("control", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": "HMAC-SHA256 " + signature
        },
        body: text,
        cache: "no-store"
      });
      if (!response.ok) {
        let detail = {};
        try { detail = await response.json(); } catch { /* keep the plain message */ }
        const code = detail && detail.error;
        throw new Error(ERROR_TEXT[code] || "The controller rejected that press (" + response.status + ").");
      }
      say(action.replace(/_/g, " ") + " applied.", "success");
    } catch (error) {
      warn(error instanceof Error ? error.message : "The press failed. Check the connection and try again.");
    } finally {
      pending = false;
      syncButtons();
    }
  }

  unlockForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const entered = tokenInput.value.trim();
    if (!entered) {
      warn("Enter the control token from Settings → Local control.");
      return;
    }
    token = entered;
    tokenInput.value = "";
    unlockForm.hidden = true;
    session.hidden = false;
    clearAlert();
    syncButtons();
  });

  forget.addEventListener("click", () => {
    token = null;
    session.hidden = true;
    unlockForm.hidden = false;
    tokenInput.value = "";
    clearAlert();
    syncButtons();
    tokenInput.focus();
  });

  for (const button of actions.querySelectorAll("button")) {
    button.addEventListener("click", () => press(button.dataset.action));
  }
  sceneForm.addEventListener("submit", (event) => {
    event.preventDefault();
    press("set_scene");
  });

  syncButtons();
  checkConnection();
})();
</script>
</body>
</html>
"####;
