<p align="center">
  <img src="assets/hypershot.svg" width="96" alt="HyperShot icon">
</p>

<h1 align="center">HyperShot</h1>

<p align="center">
  <strong>An open-source Windows screenshot tool written in Rust.</strong><br>
  Select part of your screen, mark it up, then copy, save, pin, or stitch it into a longer image.
</p>

<p align="center">
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20or%20Apache--2.0-0d9488.svg" alt="License: MIT or Apache-2.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2021-ed5a22?logo=rust&logoColor=white" alt="Rust 2021"></a>
  <a href="https://github.com/danielccustodio/hypershot/actions/workflows/ci.yml"><img src="https://github.com/danielccustodio/hypershot/actions/workflows/ci.yml/badge.svg" alt="CI workflow"></a>
</p>

<p align="center">
  <a href="#start-here">Start here</a> - <a href="#using-hypershot">Using HyperShot</a> - <a href="#building-and-testing">Build and test</a> - <a href="#contributing">Contributing</a> - <a href="README.pt-BR.md">Português (Brasil)</a>
</p>

> [!WARNING]
> **Current status:** HyperShot is a work in progress. The complete end-user experience is implemented for **Windows**: system tray, global hotkeys, the selection overlay, editing tools, saving, clipboard export, desktop pinning, and scroll stitching. The project has cross-platform capture dependencies and packaging scripts for Linux and macOS, but their native tray and overlay interface are not ready for end users. If you are looking for a ready-to-use app today, use Windows.

## What is HyperShot?

HyperShot turns a piece of your screen into an image you can immediately use. It is for the ordinary jobs that end up needing a screenshot: pointing out a bug, documenting a step, sharing an error, hiding a password before sending an image, or keeping a small reference visible while you work.

The basic flow is deliberately short:

1. Press a keyboard shortcut.
2. Drag over the part of the screen you want.
3. Add an arrow, text-like step marker, highlight, or pixelation if needed.
4. Copy it, save it, pin it, or make a longer scrolling capture.

You do not need to understand Rust to use the program. Rust is only the programming language used to build it. If you are new to developer tools, take the setup one command at a time. You will not break the project by reading the commands before you run them.

## What works today

| If you need to... | HyperShot can... |
| --- | --- |
| Capture part of the screen | Open a full-screen selection overlay and crop the chosen area. |
| Point something out | Draw with the pen, line, arrow, rectangle, or highlighter tools. |
| Show a sequence | Place numbered step badges that increase automatically. |
| Hide visible information | Pixelate a selected part of the capture. |
| Paste the image elsewhere | Copy the capture to the Windows clipboard. |
| Keep a file | Save as PNG, JPEG, or WebP through the standard save dialog. |
| Keep a reference on screen | Pin the selected image in a borderless always-on-top window. |
| Capture a tall page or chat | Build a stitched image while you scroll the selected area. |
| Launch a capture quickly | Use the tray icon, `PrintScreen`, or `Ctrl` + `Shift` + `X`. |

The current code processes captures locally. It does not contain a cloud account, an upload service, or automatic image synchronization.

## Start here

### Before you begin

For the supported Windows workflow, install these two free tools:

1. [Git](https://git-scm.com/downloads), which downloads the project.
2. [Rust](https://rustup.rs/), which provides the `cargo` command used to build and run it.

After installing Rust, close PowerShell and open it again. This gives Windows a chance to find the new `cargo` command.

### Run HyperShot from the source code

Open **PowerShell** and run the following commands one at a time:

```powershell
git clone https://github.com/danielccustodio/hypershot.git
cd hypershot
cargo run --release
```

What each command does:

- `git clone` downloads a copy of the project into a new `hypershot` folder.
- `cd hypershot` moves PowerShell into that folder.
- `cargo run --release` builds an optimized Windows executable, starts HyperShot, and opens the first capture.

The first build can take a while because Rust downloads and compiles the libraries used by the project. Later runs are usually quicker because Cargo reuses the build cache. You do not need to keep track of that cache yourself.

Once HyperShot has started, it remains available from the Windows notification area, commonly called the system tray. The small app icon is near the clock; Windows may place it behind the `^` button. If you cannot see it, do not worry, that is a normal Windows tray setting.

### If one of the commands fails

| What you see | What it usually means | What to do |
| --- | --- | --- |
| `git is not recognized` | Git is not installed or PowerShell was not reopened after installation. | Install Git, then open a new PowerShell window. |
| `cargo is not recognized` | Rust is not installed or the terminal does not yet know its location. | Install Rust through rustup, then open a new PowerShell window. |
| A capture error appears in the terminal | HyperShot could not read the screen image. | Keep the terminal message and include it in a GitHub issue, without sharing sensitive screen content. |
| `PrintScreen` does nothing | Another program may have claimed the key. | Use `Ctrl` + `Shift` + `X`, which HyperShot also registers. |

## Using HyperShot

### Take a normal capture

1. Start HyperShot once with `cargo run --release`.
2. Press `PrintScreen`. If that key is unavailable, press `Ctrl` + `Shift` + `X`.
3. Your screen freezes behind the capture overlay.
4. Click and drag to draw a rectangle around the content you want to keep.
5. Choose an editing tool from the vertical toolbar if you want to annotate the image.
6. Use the horizontal toolbar to close, save, copy, pin, or enter scroll mode.

You can also left-click the tray icon to start a capture. Right-clicking it opens a menu with capture, preferences, capture-folder, and exit options.

### Selection shortcuts

These shortcuts work while the selection overlay is open:

| Shortcut | Result |
| --- | --- |
| `Esc` | Cancel the capture. |
| `Enter` | Copy the current selection to the clipboard and close the overlay. |
| `Ctrl` + `C` | Copy the current selection to the clipboard and close the overlay. |
| `Ctrl` + `S` | Open the save dialog. |
| `Ctrl` + `Z` | Undo the last annotation. |
| Right-click, before making a selection | Cancel the capture. |

### Editing tools

The vertical toolbar provides the editing tools below. The tool names describe the intent, not a file format or a separate editing mode.

| Tool | Use it when you want to... |
| --- | --- |
| Pen | Draw a freehand mark. |
| Line | Draw a straight line. |
| Arrow | Point from one place to another. |
| Rectangle | Put a box around something. |
| Highlighter | Add a translucent colored highlight. |
| Pixelate | Obscure a rectangular area, such as a visible token or email address. Check the final image before you share it. |
| Step badge | Add numbered circles for instructions or a bug report. Each new badge uses the next number. |
| Color button | Cycle through the built-in drawing colors. It is a palette switcher, not an eyedropper. |
| Undo | Remove the latest annotation. |

The mouse wheel changes the stroke thickness while you are outside scroll-capture mode.

### Save, copy, and pin

After creating a selection:

- **Copy** puts the finished image on the clipboard. Paste it in a chat, email, document, or image editor with `Ctrl` + `V`.
- **Save** opens the Windows save dialog. You can choose PNG, JPEG, or WebP.
- **Pin** creates a small window that stays above other windows. Drag it to move it. Double-click it, or right-click it, to close it.
- **Close** discards the current capture.

When you save a capture, HyperShot can also copy it to the clipboard. This behavior is controlled by the preferences file described below.

### Capture a scrolling area

Scroll capture is useful for a long conversation, document, or web page when one screen is not enough. It works by comparing consecutive frames and joining the overlapping parts.

1. Start a normal capture and select the region you want to keep.
2. Click the **Scroll** action in the horizontal toolbar.
3. Scroll the underlying page slowly. Keep the selected region over the content you want to join.
4. When the image contains everything you need, choose **Save** to write the current stitched image, or choose **Copy**. Pressing `Enter` also copies the current stitched image and closes the overlay.

There are a few practical limits. They are worth knowing before you rely on a long capture:

- Keep some of the same content visible between scroll steps. That overlap is what lets the stitcher align images.
- Scroll slowly enough for the screen to refresh before the next frame is captured.
- Video, animated content, loading placeholders, and rapidly changing pages can prevent a clean match.
- Once scroll mode has started, drawing tools do not modify the exported stitched image. Finish annotations before entering scroll mode.

## Preferences and saved files

On Windows, right-click the tray icon and select **Edit Preferences**. HyperShot creates its configuration file automatically the first time it needs one:

```text
%APPDATA%\HyperShot\config.json
```

The file is regular JSON text. Its default settings include:

| Setting | Default | Meaning |
| --- | --- | --- |
| `save_directory` | `%USERPROFILE%\Pictures\HyperShot` | Suggested folder in the save dialog. |
| `default_format` | `png` | Initial output format for new captures. |
| `copy_to_clipboard_on_save` | `true` | Copies an image after it is saved. |
| `primary_hotkey` | `PrintScreen` | Main capture shortcut stored in the file. |
| `fallback_hotkey` | `Ctrl+Shift+X` | Alternative capture shortcut stored in the file. |
| `default_color` | red RGBA value | Starting annotation color. |
| `default_stroke_size` | `4` | Starting line thickness in pixels. |
| `enable_loupe` | `true` | Stored preference for the selection loupe. |
| `dark_backdrop_level` | `150` | Stored opacity value for the dark selection backdrop. |

The current Windows hotkeys are registered by the tray daemon as `PrintScreen` and `Ctrl` + `Shift` + `X`. Changing the text in the JSON file does not by itself change that registration yet, so do not expect a JSON edit to remap the keys.

## Command-line options

Most people can launch HyperShot with no options. These options are available when you need them:

| Command | What it does |
| --- | --- |
| `cargo run --release` | Starts the tray daemon and immediately opens a capture. |
| `cargo run --release -- --daemon` | Starts the Windows tray daemon without immediately opening a capture. `--tray` is an alias. |
| `cargo run --release -- --demo` | Captures the primary display, adds a few sample annotations, writes `hypershot_demo.png`, and attempts to copy it to the clipboard. |
| `cargo run --release -- --help` | Prints the built-in command help. |

## Building and testing

Use these commands before you open a pull request or when you need to check that your local copy still builds:

```powershell
# Run the current automated tests
cargo test --all-targets

# Check that all targets compile
cargo check --all-targets

# Create the optimized executable without starting it
cargo build --release
```

The repository includes a GitHub Actions workflow that runs `cargo check`, `cargo test`, and a release build on Windows, Ubuntu, and macOS for pushes and pull requests. You do not need a GitHub Actions account to run the same commands on your own machine.

### Build Windows distribution files locally

The project includes an Inno Setup script for creating a Windows installer and a portable ZIP. Install **Inno Setup 6** first, then run:

```powershell
powershell -ExecutionPolicy Bypass -File installer\windows\build_installer.ps1
```

The script builds the release executable, creates the installer, and places the output in `dist\`. Distribution artifacts are ignored by Git, so they are not committed accidentally.

There are also packaging scripts under `installer/linux/` and `installer/macos/`. They are part of the repository, but they do not change the Windows-only end-user support status described at the top of this document.

## Project layout

| Path | What you will find there |
| --- | --- |
| `src/main.rs` | Program entry point and command-line handling. |
| `src/core/` | Screen capture, image filters, annotation primitives, clipboard export, settings, and scroll stitching. |
| `src/ui/` | The Windows tray daemon, selection overlay, toolbars, and pinned window. |
| `assets/` | Application icons. |
| `installer/` | Installer and package scripts. |
| `scripts/` | Asset-generation helper scripts. |
| `.github/workflows/` | CI and release automation. |

## Privacy and responsible sharing

HyperShot does not automatically upload captures, but screenshots can still contain passwords, API keys, personal messages, addresses, customer data, or private documents. Pixelation is provided to help you hide a chosen region, not as a guarantee that every sensitive detail is gone.

Before sending a capture to anyone:

1. Look at the whole image, including browser tabs, notification banners, sidebars, and the taskbar.
2. Pixelate or crop information you do not intend to share.
3. Reopen the saved image and check it once more if it will be public.
4. Never post secrets, access tokens, or private screenshots in a GitHub issue or pull request.

## Contributing

Contributions are welcome, including from first-time contributors. You do not need to arrive with a large feature. Documentation improvements, clear bug reports, focused tests, Windows testing, and work toward Linux or macOS support are all useful.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) before starting. A simple contribution workflow looks like this:

1. Fork the repository on GitHub.
2. Create a branch with a short, descriptive name.
3. Make one focused change.
4. Run `cargo test --all-targets`.
5. Open a pull request that says what changed, why it changed, and how you tested it.

If you are reporting a bug, include the Windows version, the command you ran, the exact error message, and steps another person can follow. Remove private data from screenshots and logs first.

## License

HyperShot is available under either the [MIT License](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.
