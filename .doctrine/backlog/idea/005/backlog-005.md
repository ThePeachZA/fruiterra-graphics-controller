# IDE-005: Overlay refresh control

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Owner request, 2026-11-21.

## Problem

The owner listed "Overlay refresh" without further detail. Capture the intent
now; clarify the behaviour before implementation.

## Possible scope

- An explicit **Refresh overlays** action that asks OBS Browser Sources to
  reload, or a cache-busting URL variant.
- An automatic refresh rule tied to asset changes or server restart.

## Constraints

- `docs/backlog.md` C3 already records that assets are content-addressed, so a
  changed image gets a new URL and cannot show a stale file. Any refresh work
  must explain what problem remains beyond that.
- A9 fixed stale team logos in the frozen overlay; do not reintroduce that bug.
- No schema, protocol, or frozen overlay changes without a separate decision.
- Do not claim this is a legacy import fix or a finished item.

## Clarification needed

- Manual button, automatic on change, or both?
- Refresh one overlay, the selected set, or all Browser Sources?
- What observable QA evidence proves the refresh happened?
