# 📋 ClipCraft Studio (Desktop) — Project Roadmap & Tickets

Fork dari `storytold/filmcraft` yang dimodifikasi menjadi Studio NLE Mandiri dengan AI Auto-Clipper terintegrasi untuk produksi video vertikal 9:16 (Shorts/TikTok/Reels) dan video landscape sinematik.

---

## 🎯 Architecture Overview
- **Base Engine**: Pure Rust Non-Linear Editor (`crates/engine`, `crates/render`, `crates/gpu`)
- **GPU Compositor**: `wgpu` (Metal, Vulkan, DirectX 12)
- **GUI Framework**: `egui` via `crates/ui-egui`
- **Control Interface**: Loopback TCP JSON lines protocol (`127.0.0.1:9876`) + MCP Server
- **AI Integration**: Connected to `clipcraft-api` (FastAPI backend) via async HTTP/SSE client
- **Licensing**: Dual MIT / Apache-2.0 (Clean-room rebrand, zero proprietary ArtCraft assets)

---

## 📌 Tickets Breakdown

### [DESKTOP-01] Brand Sanitization & Project Rebrand
- [x] Remove all ArtCraft proprietary logos, trademarks, and branding assets in accordance with MIT/Apache-2.0 notice.
- [x] Rebrand binary identifiers, package names (`Cargo.toml`), and window titles to `ClipCraft Studio`.
- [x] Set application identifier: `id.ezuuk.clipcraft`.
- [x] Add new custom vector/SVG application icons.

### [DESKTOP-02] UI Theme & Styling Preservation
- [x] Retain FilmCraft's native dark slate/obsidian palette in `crates/ui-egui/src/theme.rs`.
- [x] Absorb theme design tokens (margins, font hierarchies, panel frames) into internal guidelines.
- [x] Ensure consistent aesthetic harmony across all stock and custom panels.

### [DESKTOP-03] Custom Panel: AI Clipper (`ai_clipper.rs`)
- [x] Create dockable panel module in `crates/ui-egui/src/panels/ai_clipper.rs`.
- [x] Register panel in dock registry (`crates/ui-egui/src/dock.rs` and `panels/mod.rs`).
- [x] Design panel UI components:
  - Input field: YouTube URL / Local media selection.
  - Dropdown options: Content Genre (`Podcast Duo`, `Solo Creator`, `Gaming Stream`).
  - Threshold slider: Virality Score cut-off (default: `90`).
  - Action button: `[⚡ Auto-Generate Shorts]`.
  - Real-time progress bar binding to `clipcraft-api` SSE stream.
  - Card list of viral moments (Thumbnail, Hook title, Virality badge, Start/End times).
  - Action buttons per card: `[Add to Sequence]` and `[Direct Quick Export]`.

### [DESKTOP-04] "Shorts AI" Workspace Preset (9:16 Vertical)
- [x] Register new workspace preset in `crates/ui-egui/src/dock.rs` alongside Assembly, Editing, Color, and Effects.
- [x] Configure default layout:
  - Left: Project Bin & Source Monitor.
  - Center: Program Monitor locked to 9:16 aspect ratio (`1080x1920`).
  - Right: Docked AI Clipper panel.
  - Bottom: Multi-track Timeline (V1/V2 video tracks, A1/A2 audio, C1 caption track).

### [DESKTOP-05] Engine Command Bridge for Automated Timeline Ingestion
- [x] Wire panel events to native engine commands via `filmcraft_engine::Session::execute`.
- [x] Implement automated sequence generator:
  - Create 9:16 sequence (`file.newSequence` with 1080x1920 resolution, 30fps).
  - Add subtitle track (`captions.newTrack` and `captions.setStyle`).
  - Set sequence status feedback in application UI.

### [DESKTOP-06] Client-Side HTTP & SSE Bridge to `clipcraft-api`
- [x] Implement lightweight async HTTP client using `ureq` in `crates/ui-egui/src/panels/ai_clipper.rs`.
- [x] Connect to production FastAPI backend (`https://clipper.ezuuk.my.id`).
- [x] Handle streaming Server-Sent Events (SSE) for non-blocking UI updates and background polling.

### [DESKTOP-07] Multi-Platform Native Packaging & Release
- [x] Setup release build matrix for macOS (arm64 Apple Silicon & x86_64 universal).
- [x] Setup Windows release packaging (x64 ZIP & binaries).
- [x] Setup Linux packaging (x86_64 Tarball & binaries).
- [x] Configure automated multi-platform release script (`packaging/release-matrix.sh`).
