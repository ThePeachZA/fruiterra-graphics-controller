# Assets

The Assets page is the image library used by teams, talent, matches and
overlays. Images are stored by content hash, so importing the same file twice
does not duplicate it.

## Categories

Choose a **Category** before importing:

| Category | Used for |
|---|---|
| **Logo** | Team and tournament logos |
| **Portrait** | Player and talent portraits |
| **Map** | Map images |
| **Hero** | Hero portraits and icons |
| **Game mode** | Game-mode icons |
| **Role** | Role icons |
| **Side** | Attack and defend icons |

## Import an image

### From a file

1. Choose the **Category**.
2. Select **Import file…**.
3. Pick a PNG, JPEG, WebP, GIF or SVG file.

### From a URL

1. Choose the **Category**.
2. Select **Import from URL…**.
3. Paste the image address and select **Import**.

The limit is **20 MB** per image. SVG files are checked for scripts and external
references before they are stored; unsafe files are rejected.

Bundled art ships with the application and is read-only. Imported images are
content-addressed and read-only once stored. You can rename an asset, but the
image data and its URL do not change.

## Find an image

- Type in **Search assets by name**.
- Use the category chips to filter. Each chip shows its count.
- Select an image to preview it.
- Use **Rename** to change only the display name.
- Use **Delete** to remove an image you imported.

## Project bundles

A project bundle moves a project and its referenced images to another machine.

- **Export project…** asks for a folder and creates
  `fruiterra-project-<timestamp>/` inside it. The bundle contains `app.json`,
  `export.json` and the referenced user images. Bundled art is not copied.
- **Import project…** asks for a bundle folder and replaces the current project
  after confirmation.

Use a bundle when moving to a new PC or before a risky change. See
[Data and Backups](Data-and-Backups).

## Unused image data

Unreferenced image data is deleted automatically after a successful save and on
startup. To clean up immediately, open **Settings → Asset storage** and select
**Clean up now**. The panel shows how many files and how much space would be
removed.

Only user-imported, unreferenced images are removed. Bundled assets and images
still used by the saved or current project are kept.

## Where images live

```text
%APPDATA%\Fruiterra\Fruiterra Graphics Controller\assets\blobs\
```

Do not rename or edit files in this folder. Use the app to import and manage
images.

## Related pages

- [Teams and Talent](Teams-and-Talent)
- [Settings](Settings)
- [Data and Backups](Data-and-Backups)
