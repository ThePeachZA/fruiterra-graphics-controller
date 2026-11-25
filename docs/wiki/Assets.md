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
to replace, rename or delete.

### From an image picker

Image pickers in Teams, Talent and Match Controller can upload a new image
directly into the relevant category. When a picker upload creates a new asset,
that asset is selected automatically.

## Replace an image

Select **Replace…** on an imported asset card, or open its preview and choose
**Replace…**. Pick the new PNG, JPEG, WebP, GIF or SVG file when the system file
dialog opens. The asset keeps its name, category and identity, so every team,
player and overlay that uses it follows the new image without being linked
again. The old image data becomes unused and is removed after the next save.

Bundled assets and images attached to a hero cannot be replaced. The button is
disabled while a replacement is being read and stored.

## Rename or delete

- Select **Rename** on an imported asset to change its display name.
- Select **Delete** to remove an imported asset. If something still uses it, the
  reference is cleared rather than left broken.

Bundled assets cannot be renamed or deleted.

## Hero and map properties

Some assets have properties that tell the overlay which variant to use.

- A **Hero** asset has **Properties…**. It has three optional attachments:
  - **Ban icon** - a small image used where the hero is banned. The button
    becomes **Replace ban icon…** once one exists, and **Remove ban icon**
    clears it. Without one, the hero-bans overlay uses the hero's own art.
  - **Transparent art** - a cut-out of the hero on a transparent background.
    The hero-bans overlay prefers it over the normal art and stands it on the
    bottom of the screen. Without one, the normal art is used.
  - **Badge** - a small hexagonal emblem for the hero, shown next to the role.
  - **Role** - Tank, Damage or Support. The hero-bans overlay draws the role
    as a hexagon next to the hero's name.
  Each attachment is owned by that hero: it is hidden from the library, hero
  pickers and ban lists, and is deleted when you replace or remove it, or when
  you delete the hero. The previews update as soon as a replacement is chosen,
  without closing the dialog.
- A **Map** asset has an inline **Game mode** select. The map-pool overlay groups
  maps by that mode, and Match Controller fills **Game mode** on map slots that
  use the map. Choose **No game mode** to clear it.

Changing a map's game mode refreshes the **Game mode** field on map slots that
still show the derived value. If you type a mode by hand, that value is kept
until you choose a different map name, which re-derives the mode from the map's
current game mode.

## Browsing the library

The library opens with the assets themselves, not a wall of panels.

- The **search** box and the **Sort assets** list sit on the toolbar, with the
  category chips underneath. Sort by **Name (A-Z)**, **Name (Z-A)**,
  **Category**, **Newest first** or **Oldest first**.
- The library is **paged**, 24 assets at a time, so a large collection loads
  quickly. The pager under the grid shows what is on screen and how many pages
  there are.
- **Import files…**, **Import from URL…** and the import **Category** sit on one
  row under the toolbar.
- Thumbnails are fetched from the running overlay server, so the application
  never holds a copy of every image in memory. A placeholder shows while a
  thumbnail loads. If the overlay server is stopped, the image is inlined
  instead.

## Project backup

A project backup copies the project and the user images it references to
another machine. It lives in **Settings → Project backup**.

- **Export project…** writes a folder named `fruiterra-project-<unix-timestamp>`
  containing `app.json`, `export.json` and the referenced blobs.
- **Import project…** replaces the current project with the bundle. This cannot
  be undone.

Bundled art is never copied because every installation already has it.

A project backup and a **content pack** are different things. A backup moves
*your* project - teams, matches, settings and images. A pack carries artwork
only, so installing one adds images and changes nothing else.

## Clean up storage

Unused imported image data is removed automatically after a successful save and
on startup. In **Settings → Asset storage**, **Clean up now** removes anything
the saved and current projects no longer use.

## Related pages

- [Teams and Talent](Teams-and-Talent)
- [Match Controller](Match-Controller)
- [Data and Backups](Data-and-Backups)
