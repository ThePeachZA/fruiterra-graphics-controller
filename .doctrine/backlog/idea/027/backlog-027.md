# IDE-027: Legacy Electron app.json importer

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Migrated from the “can be added later” backlog note in `docs/decisions.md` and the scope guard in `docs/implementation-plan.md` §8.

Import the earlier Google Sheets/Electron `app.json` by mapping legacy JSON into the current domain model. No importer shipped in v1 and no automatic import is documented in the README.

**Status: triaged.** Declined for v1; kept as future work without changing the domain model first.
