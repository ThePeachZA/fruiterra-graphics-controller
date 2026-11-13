# FAQ

### Is this the same as the old Google Sheets workflow?

It replaces it with a desktop app. The overlay protocol is unchanged, so overlay
HTML from the old setup still works. There is no automatic import of the old
spreadsheets; teams, talent, matches and assets are entered in the app.

### Can I keep my existing overlays?

Yes. The seven bundled overlays are drop-in compatible with the previous
protocol, and third-party overlays can connect with `?port=<port>`. If you edited
an overlay, put your copy in the **Overlays folder** shown in Settings.

### Is there a timer overlay?

No. The timer is control-only in version 1. Third-party overlays still receive
`matchTimer` ticks and the start, pause and reset actions.

### Is there a bracket overlay?

Not in version 1. The bracket board is for the operator; it is not a broadcast
overlay.

### Can I run more than one match or scene?

The Match Controller edits one current match at a time. There are no presets,
multiple scenes or undo/redo in version 1.

### Can I use Stream Deck or an HTTP API?

No. Version 1 has no HTTP control API and no Stream Deck integration.

### Does LAN mode require a password?

No. There is no authentication in version 1. Use **Allow LAN connections** only
on a trusted network, and turn it off when you are done.

### Why does Windows warn about the app?

Version 1 is unsigned. Verify the download against `SHA256SUMS.txt`, then choose
**More info → Run anyway** if you trust the source.

### Does it update itself?

No. Download a newer release and install it over the old version. Your project
data is kept.

### Does it run on macOS or Linux?

No. Version 1 is Windows x64 only.

### Where is my data?

In `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\`. See
[Data and Backups](Data-and-Backups) for the folder layout and how to move it.

### How do I move to another PC?

Export a project bundle from **Assets**, copy the folder to the new PC, install
the app there and select **Import project…**.

### Why do the overlays use a different font?

Version 1 bundles Archivo under the neutral `FGC Display*` family names. It is
licensed for redistribution and served locally, so there are no runtime font
downloads. The layout and protocol are unchanged. The typeface is not Apotek,
and Saira remains bundled while the final font choice is reviewed.

### Can I add custom points, match history or multi-stage playoffs?

Not in version 1. These are recorded as post-v1 work. The current release covers
single- and double-elimination brackets, round-robin groups, tiebreakers and
group-to-playoff generation.

### Where do I report a bug?

Open an issue at
[github.com/ThePeachZA/fruiterra-graphics-controller/issues](https://github.com/ThePeachZA/fruiterra-graphics-controller/issues).
Include the app version, your Windows and OBS versions, the steps to reproduce,
and the newest log file.
