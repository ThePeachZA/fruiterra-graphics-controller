# Installation

Version 1 runs on **Windows 10 or Windows 11, 64-bit only**.

## Choose a build

| Build | Use it when |
|---|---|
| `fruiterra-gc-<version>-windows-x64-setup.exe` | You want a normal install with a Start Menu entry and uninstaller. |
| `fruiterra-gc-<version>-windows-x64-portable.zip` | You want to run the app from a folder without installing anything. |

Both are on the [Releases page](https://github.com/ThePeachZA/fruiterra-graphics-controller/releases),
together with `SHA256SUMS.txt`.

## Verify the download

Releases are unsigned in version 1, so Windows SmartScreen may warn on first
run. Before running either file, compare its SHA-256 hash with the value in
`SHA256SUMS.txt`:

```powershell
Get-FileHash .\fruiterra-gc-<version>-windows-x64-setup.exe -Algorithm SHA256
```

If the hashes do not match, delete the file and download it again.

## Install with the installer

1. Run `fruiterra-gc-<version>-windows-x64-setup.exe`.
2. If SmartScreen appears, choose **More info**, confirm the app name and
   publisher, then choose **Run anyway**.
3. Follow the installer. It installs for the current user, so administrator
   rights are not normally required.
4. If the WebView2 runtime is missing, the installer adds it.
5. Launch **Fruiterra Graphics Controller** from the Start Menu.

The installer creates a Start Menu entry and an uninstall entry in Windows
Settings.

## Use the portable build

1. Extract the ZIP to a folder you can write to, for example
   `C:\Tools\Fruiterra Graphics Controller\`.
2. Keep the `assets\` folder beside `fruiterra-gc.exe`.
3. Run `fruiterra-gc.exe`.

The portable build does not install anything and does not write registry keys.
It **requires the WebView2 runtime to be present already**. If it is missing,
the app cannot start; install the Evergreen WebView2 runtime or use the
installer instead.

> [!IMPORTANT]
> Do not run `fruiterra-gc.exe` from inside the ZIP preview. Extract the whole
> folder first, including `assets\`.

## First launch

The app creates its data folders on first launch:

```text
%APPDATA%\Fruiterra\Fruiterra Graphics Controller\
```

No project is loaded yet. Continue with [Quick Start](Quick-Start).

## Update

Install the newer build over the old one. Your project data is stored outside
the install folder, so it is kept. Close the app before installing an update.

For the portable build, replace the old folder with the new one. Keep your data
folder untouched.

## Uninstall

Use **Settings → Apps → Installed apps → Fruiterra Graphics Controller →
Uninstall**, or the uninstaller in the Start Menu folder.

The uninstaller removes the application and its shortcuts. Your project data is
kept in `%APPDATA%\Fruiterra\Fruiterra Graphics Controller\`. If you also want
to remove that, export a project bundle first (see
[Data and Backups](Data-and-Backups)), then delete the folder manually.

## Related pages

- [Quick Start](Quick-Start)
- [Overlays in OBS](Overlays-in-OBS)
- [Troubleshooting](Troubleshooting)
