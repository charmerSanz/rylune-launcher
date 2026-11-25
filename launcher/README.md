# Rylune Client 0.9 — Native
Native Rust desktop launcher using eframe/egui (OpenGL), Rust download and Minecraft process management, and existing Crimson artwork. No Node, React, Tauri or WebView frontend.

## Build
Install Rust stable MSVC and Visual Studio C++ build tools with Windows SDK. From launcher, run cargo run or cargo build --release. The GitHub Actions workflow builds RyluneClient.exe and an NSIS installer. Java is required by Minecraft itself and automatically installed from official metadata if missing.

Existing %APPDATA%/Rylune/settings.json and profiles directories are reused. Local accounts are for Minecraft owners' local play; Microsoft sign-in requires an approved application ID. Offline installations use official public endpoints and require internet for missing files. Minecraft's official catalog supplies available versions in the 1.21.11–26.3 range. Every profile retains its own files.

The UI uses background Rust worker threads for downloads and authentication, and repaint notifications on events. It is not an FPS mod. First-run downloads are still sequential; gameplay and Microsoft sign-in need actual Windows runtime testing. Automated tests include official metadata and Java-process checks. The installer is unsigned and does not remove profile data.
