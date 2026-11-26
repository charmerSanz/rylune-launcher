# Rylune Client 0.12 — Native
Native Rust desktop launcher using eframe/egui (OpenGL), Rust download and Minecraft process management, and existing Crimson artwork. No Node, React, Tauri or WebView frontend.

## Build
Install Rust stable MSVC and Visual Studio C++ build tools with Windows SDK. From launcher, run cargo run or cargo build --release. The GitHub Actions workflow builds RyluneClient.exe and an NSIS installer. Java is required by Minecraft itself and automatically installed from official metadata if missing.

Existing %APPDATA%/Rylune/settings.json and profiles directories are reused. Local accounts are for Minecraft owners' local play; Microsoft sign-in requires an approved application ID. Offline installations use official public endpoints and require internet for missing files. Minecraft's official catalog supplies available versions in the 1.21.11–26.3 range. Every profile retains its own files.

The UI uses background Rust worker threads for downloads and authentication, and repaint notifications on events. It is not an FPS mod. First-run downloads are still sequential; gameplay and Microsoft sign-in need actual Windows runtime testing. Automated tests include official metadata and Java-process checks. The installer is unsigned and does not remove profile data.

## Obsidian interface
Near-black surfaces, rounded texture meshes, raised crimson buttons, animated hover and navigation, and a short startup logo reveal. Reduce animations skips the opening sequence and disables interpolated motion. No new renderer or runtime dependencies; the existing game backend remains unchanged.

## Content
Discover searches Modrinth for the active Minecraft version and Fabric. Mods, resource packs and shaders install into the selected profile with SHA-512 verification and required dependency resolution. Library scans local files and supports disable/enable, recoverable removal, import and managed updates. Modpacks create separate profiles and pin Fabric Loader; local .mrpack imports are supported. Optional pack files are skipped, unsupported Minecraft versions/loaders and protected paths rejected. Content installs are blocked while a game is running. Downloads use HTTPS and a restricted host list. Only managed files can be updated automatically; optional/incompatible dependency resolution and full pack upgrades are not implemented. Only install trusted mods: JARs execute code inside Minecraft.
