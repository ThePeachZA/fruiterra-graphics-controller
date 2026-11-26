# Touch Portal

Touch Portal has a built-in **Http Post** action with a URL, data, authorization
header, content type and custom headers. It can post a fixed JSON body, but it
does not generate a new Unix timestamp and nonce for each press. A fixed body is
rejected as a replay after its first use.

Use an **Execute Script** action with the supplied wrapper instead. The script
reads the current token from disk and creates fresh replay values every time.
This is a setup guide with a helper script, not an importable `.tpz` file.

## Files

Two files live in `examples/control/scripts/`:

- `control-touch-portal.ps1` builds and sends the request.
- `control-touch-portal.cmd` calls PowerShell with an execution-policy bypass.

Copy both files to a stable local folder, such as
`%USERPROFILE%\Documents\Fruiterra Control\`. Keep the folder readable only by
the Windows account that runs Touch Portal. Do not put the token in the command
line.

The `.cmd` wrapper uses `-ExecutionPolicy Bypass` because a machine policy may
otherwise block a downloaded PowerShell script. Only use it with a script you
have reviewed and stored locally.

## Add a button

1. In Touch Portal, open the page that will hold the control buttons.
2. Add a button and set its event to the press action you want, normally
   **On Pressed**.
3. Add the action **Execute Script**.
4. Set **Filepath** to the full path of `control-touch-portal.cmd`, for example:

   ```text
   C:\Users\<you>\Documents\Fruiterra Control\control-touch-portal.cmd
   ```

5. Set **Arguments** to the action for that button. Examples:

   ```text
   -Action show
   -Action hide
   -Action force_update
   -Action next_scene
   -Action previous_scene
   -Action set_scene -Scene game-overlay.html
   ```

6. Add `-Port <port>` only if the overlay server is not using `3001`.
7. Repeat for each action. Give each button a clear name; Touch Portal does not
   infer meaning from the command line.

The built-in **Http Post** action is still useful for a one-off diagnostic with
a hand-generated nonce, but it should not be used for repeated show buttons.

## What the script does

The PowerShell script:

- accepts one of the six supported actions and an optional scene and port;
- rejects `set_scene` without a scene;
- reads the token from `FGC_DATA_DIR\data\control-token` when that variable is
  set, otherwise from
  `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\data\control-token`, with a
  debug-build fallback to the `(dev)` data root;
- creates a current Unix timestamp and a unique nonce from the timestamp,
  process id and a GUID;
- builds compact JSON with `ConvertTo-Json`;
- posts to `http://127.0.0.1:<port>/control` with the bearer header and prints a
  short status line.

The token never appears in the URL, arguments or normal output. If an HTTP call
fails, PowerShell's error text may include request details, so do not publish
Touch Portal logs without checking them.

## Verification

Press each button once while the overlay server is running. The expected result
is the matching show, hide, scene or update behaviour in OBS. An error dialog or
red key state usually means one of the following:

| Symptom | Fix |
|---|---|
| `control_unavailable` | Start the server from the app. |
| `unauthorised` | Copy the current token from **Settings → Local control**; regeneration invalidates old copies. |
| `loopback_only` | Run Touch Portal and the script on the host computer. |
| `replayed_nonce` | Press again; every script run creates a new nonce. |
| `rate_limited` | Wait for the one-minute window. |
| Script cannot find the token | Check `FGC_DATA_DIR`, `--data-dir`, and whether this is a debug `(dev)` data root. |
| PowerShell says the script is blocked | Confirm the `.cmd` wrapper is being used and that the script is a local file you trust. |

No `.tpz` profile is supplied. Build and test the buttons in your own Touch
Portal installation so the profile stays tied to your local paths and token
handling.

## Related pages

- [Local control](../../docs/wiki/Local-Control.md)
- [Control examples](README.md)
- [Stream Deck with API Monkey](stream-deck-api-monkey.md)
