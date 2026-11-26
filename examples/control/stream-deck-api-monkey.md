# Stream Deck with API Monkey

API Monkey (`ft-t/apimonkey`, MIT licence) can send a POST request from a Stream
Deck key and run Lua when the key is pressed. This guide builds one key at a
time. It is not an importable `.streamDeckProfile`; profile formats vary between
plugin versions, and the token must be configured locally anyway.

The key runs on the computer that hosts Fruiterra Graphics Controller. The
plugin cannot reach the control endpoint from a different machine, even when
that machine can load remote overlays.

## Install and create the key

1. Install API Monkey from the Stream Deck store.
2. Drag **API Monkey** onto a free key.
3. Set **Request Type** to `POST`.
4. Set **API URL** to the exact endpoint shown in **Settings → Local control**,
   for example `http://127.0.0.1:3001/control`.
5. Set **Polling** to `0`. This example sends only when the key is pressed.
6. Add these parameters in the plugin's **Parameters** section:

   | Name | Value |
   |---|---|
   | `Token` | The bearer token from **Settings → Local control**. |
   | `Action` | The action for this key, for example `show`. |
   | `Scene` | The scene file name, only for a `set_scene` key. |

7. Add a header with name `Content-Type` and value `application/json`. You do
   not need an `Authorization` header here; the Action Lua below adds it with a
   fresh value.
8. Set **Button Action** to **Execute Lua**.
9. Paste the script below into **Action Lua Script**.
10. Select **Save All**. Changes are not applied until you do.

Create one key per action. Use `Scene` only on a `set_scene` key; other keys can
leave it empty.

## Action Lua script

```lua
-- API Monkey Action Lua for POST /control.
-- GopherLua / Lua 5.1; json and http are preloaded by the plugin.

local allowed = {
  show = true,
  hide = true,
  force_update = true,
  set_scene = true,
  next_scene = true,
  previous_scene = true,
}

local action = ButtonConfig.parameters.Action
if type(action) ~= "string" or not allowed[action] then
  error("Set the Action parameter to a supported control action")
end

local token = ButtonConfig.parameters.Token
if type(token) ~= "string" or token == "" then
  error("Set the Token parameter from Settings -> Local control")
end

local payload = {
  action = action,
  timestamp = os.time(),
  nonce = string.format(
    "%08x%08x%08x",
    os.time(),
    math.floor(os.clock() * 1000000),
    math.random(0, 0x7fffffff)
  ),
}

if action == "set_scene" then
  local scene = ButtonConfig.parameters.Scene
  if type(scene) ~= "string" or scene == "" then
    error("Set the Scene parameter for set_scene")
  end
  payload.scene = scene
end

local body, encode_error = json.encode(payload)
if not body then
  error("Could not encode the control request: " .. tostring(encode_error))
end

local response = http.request({
  method = "POST",
  url = ButtonConfig.apiUrl,
  headers = {
    Authorization = "Bearer " .. token,
    ["Content-Type"] = "application/json",
  },
  body = body,
  timeout_seconds = 5,
})

if response.status_code ~= 200 then
  error("Control request failed with HTTP " .. tostring(response.status_code))
end

return "OK"
```

The script returns a scalar string, which API Monkey can place in the key title
through **Title Prefix** and **Response Mapping**. It does not return the token
or the response body.

## Why the request is built in Lua

API Monkey renders Go templates in its saved URL, body and header values before
it sends a request. That is useful for fixed parameters, but the control API
needs a new timestamp and nonce for every press. Action Lua runs at press time,
so it can generate both values and build the JSON body itself.

`http.request` accepts string header values only. The script therefore builds
`Authorization` as one string. HTTP 4xx and 5xx responses are returned normally,
so the script checks `status_code` explicitly and raises an error for anything
other than 200. A Lua error makes Stream Deck show the key alert and records the
failure in the plugin log.

## Operational notes

- Only one API Monkey key action runs at a time. Presses made while a request is
  running are ignored; they are not queued.
- The token is stored in the Stream Deck profile configuration. Protect that
  profile and the computer it runs on. Regenerate the token in Settings if it
  may have been copied.
- If a press returns `replayed_nonce`, press again. The new execution creates a
  new nonce.
- If a `set_scene` key returns success but nothing changes, confirm that scene
  is included in the current auto-overlay set.
- Plugin logs are in `logs/log.log` inside the installed API Monkey plugin
  directory. Start there when a key shows a red alert.
- This example does not use TLS verification options. The endpoint is loopback
  HTTP; there is no certificate to verify.

## Related pages

- [Local control](../../docs/wiki/Local-Control.md)
- [Control examples](README.md)
- [Touch Portal](touch-portal-plugin.md)
