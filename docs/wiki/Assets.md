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

### From files

1. Choose the **Category**.
2. Select **Import files…**.
3. Pick one or more PNG, JPEG, WebP, GIF or SVG files.

If some files fail to import, the successful imports are kept and the errors are
reported.

### From a URL

1. Choose the **Category**.
2. Select **Import from URL…**.
3. Paste the image address and select **Import**.

The limit is **20 MB** per image. SVG files are checked for scripts and external
references before they are stored; unsafe files are rejected.

Bundled art ships with the application and is read-only. Imported art is yours
to rename or delete.

### From an image picker

Image pickers in Teams, Talent and Match Controller can upload a new image
directly into the relevant category. When a picker upload creates a new asset,
that asset is selected automatically.

## Rename or delete

- Select **Rename** on an imported asset to change its display name.
- Select **Delete** to remove an imported asset. If something still uses it, the
  reference is cleared rather than left broken.

Bundled assets cannot be renamed or deleted.

## Hero and map properties

Some assets have properties that tell the overlay which variant to use.

- A **Hero** asset has **Properties…**. Use **Upload ban icon…** to attach a
  small image to that hero; the button becomes **Replace ban icon…** once one
  exists, and **Remove ban icon** clears it. The **Ban icon** preview updates as
  soon as a replacement is chosen, without closing the dialog. The ban icon is
  owned by that hero: it is hidden from the library, hero pickers and ban lists,
  and is deleted when you replace/remove it or delete the hero. Without one, the
  hero-bans overlay uses the hero's own art.
- A **Map** asset has an inline **Game mode** select. The map-pool overlay groups
  maps by that mode, and Match Controller fills **Game mode** on map slots that
  use the map. Choose **No game mode** to clear it.

Changing a map's game mode refreshes the **Game mode** field on map slots that
still show the derived value. If you type a mode by hand, that value is kept
until you choose a different map name, which re-derives the mode from the map's
current game mode.

Library thumbnails load lazily and are cached by content hash, so revisiting the
page does not decode every image again. Long asset names show a tooltip and
wrap to at most two lines.

## Project bundles

A project bundle copies the project and the user images it references to another
machine.

- **Export project…** writes a folder named `fruiterra-project-<unix-timestamp>`
  containing `app.json`, `export.json` and the referenced blobs.
- **Import project…** replaces the current project with the bundle. This cannot
  be undone.

Bundled art is never copied because every installation already has it.

## Clean up storage

Unused imported image data is removed automatically after a successful save and
on startup. In **Settings → Asset storage**, **Clean up now** removes anything
the saved and current projects no longer use.

## Related pages

- [Teams and Talent](Teams-and-Talent)
- [Match Controller](Match-Controller)
- [Data and Backups](Data-and-Backups)
