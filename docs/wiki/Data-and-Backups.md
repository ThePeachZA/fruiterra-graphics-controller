# Data and Backups

The app keeps everything in your Windows user profile, outside the install
folder.

## Folder layout

```text
%APPDATA%\Fruiterra\Fruiterra Graphics Controller\
├── data\
│   ├── app.json
│   └── app.json.bak
├── assets\blobs\
├── overlays\
└── logs\
```

| Path | Contents |
|---|---|
| `data\app.json` | The saved project: teams, talent, matches, tournaments, assets and settings. |
| `data\app.json.bak` | The previous good save. Used automatically if `app.json` is unreadable. |
| `assets\blobs\` | Imported images, stored by content hash. |
| `overlays\` | Optional per-overlay HTML overrides. |
| `logs\` | Daily application logs. |

Do not edit `app.json` by hand unless you are recovering a backup. Close the app
first.

## Save behaviour

- Changes save automatically after a short delay.
- The app keeps one previous good save as `app.json.bak`.
- If `app.json` is corrupt or unreadable, the app tries the backup.
- A save file from a newer app version is not loaded. The app starts with an
  empty project and shows an error instead. Update the app before making
  changes: the first save replaces the file and keeps the newer one as
  `app.json.bak`, and a later save replaces that backup too.

## Back up

### Recommended: export a project bundle

1. Open **Assets**.
2. Select **Export project…**.
3. Choose a folder outside the app data folder.

This creates `fruiterra-project-<timestamp>/` with `app.json`, `export.json` and
the referenced user images. Bundled art is not copied because it ships with the
app.

### Full manual copy

To copy everything, close the app first, then copy the whole
`Fruiterra Graphics Controller` folder. This includes logs and unused image
data.

## Restore

### From a project bundle

1. Open **Assets**.
2. Select **Import project…**.
3. Choose the bundle folder.
4. Confirm the replacement.

The current project is replaced.

### From a manual copy

1. Close the app.
2. Rename the current data folder, for example to
   `Fruiterra Graphics Controller.old`.
3. Copy your backup folder into its place, keeping the same name.
4. Start the app.

If the app starts with an empty project, it created a new data folder. Close it
and check the folder name and path.

## Move to another PC

1. On the old PC, export a project bundle.
2. Copy the bundle folder to the new PC.
3. Install the app on the new PC.
4. On the new PC, open **Assets → Import project…** and choose the bundle.

This moves the project and every referenced user image.

## Logs

Logs are written to the `logs\` folder. They are useful when reporting a problem:
include the newest log file with the report. Daily logs are kept to seven files.

## Reset the project

There is no in-app reset button.

1. Close the app.
2. Rename or move the `Fruiterra Graphics Controller` folder.
3. Start the app.

A fresh folder and an empty project are created. Keep the old folder until you
are sure you do not need it.

## Legacy data

There is no automatic import from the earlier Google Sheets-based workflow. The
app does not read the old spreadsheets. Teams, talent, matches and assets must be
re-entered or imported from a project bundle created by this app.

## Related pages

- [Assets](Assets)
- [Installation](Installation)
- [Troubleshooting](Troubleshooting)
