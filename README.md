# Certificate Studio 

A high-performance native desktop certificate designer and bulk generator written in **Rust** using **`egui` / `eframe`**.

Designed for educators, event organizers, and institutions to visually lay out certificate templates and generate personalized certificates with custom typography and millimeter-accurate document rendering.

---

## Features

- **Native Desktop GUI**: Built directly in Rust with `egui` and `eframe` (no Electron, webviews, or browser overhead).
- **Physical Document Coordinates**: Document layouts and positions are stored strictly in **millimeters** (`mm`), ensuring identical visual fidelity between the canvas editor, PNG exports, and PDF printouts.
- **Certificate Background Import**: Native file dialog (`rfd`) to load PNG, JPG, or WebP certificate designs while preserving exact page aspect ratios.
- **Interactive 8-Point Element Resizing & Moving**: Click and drag text boxes freely across the canvas, and scale with 8 directional resize handles.
- **Custom Font Management**: Load and validate custom `.ttf` and `.otf` fonts using `fontdue`, with live egui font switching per text element.
- **Full Undo / Redo**: Comprehensive snapshot-based history supporting `Ctrl+Z` (Undo) and `Ctrl+Y` / `Ctrl+Shift+Z` (Redo).
- **Page Presets**: Supports A4 Landscape (297 × 210 mm), A4 Portrait (210 × 297 mm), and custom millimeter dimensions.
- **Precision Status Bar**: Live pointer coordinates (`X, Y mm`), zoom factor, and background asset dimensions.

---

## Building the Studio.

### Prerequisites

- [Rust Toolchain](https://rustup.rs/) (Rust 1.80+ or latest stable)
- On Windows with GCC/MinGW (e.g. MSYS2 `ucrt64`):
  ```powershell
  rustup default stable-x86_64-pc-windows-gnu
  ```

### Build & Run

Clone the repository and launch the app:

```powershell
cargo run
```

Or build an optimized release binary:

```powershell
cargo build --release
```
The executable will be located at `target/release/certificate-studio.exe`.

### Running Tests

```powershell
cargo test
```

---

## 🗺 Roadmap

- [x] **Milestone 1**: Application foundation, native `eframe` window, layout shell, background image loader.
- [x] **Milestone 2**: 8-handle element resizing, custom TTF/OTF font loading with `fontdue`, full undo/redo stack.
- [ ] **Milestone 3**: Project persistence (`.certproj` Serde JSON) with bundled font/image asset directories.
- [ ] **Milestone 4**: CSV import, placeholder replacement (`{{name}}`, `{{event}}`), record preview.
- [ ] **Milestone 5**: PDF/PNG export engine (single PNG, single PDF, combined multi-page PDF).
- [ ] **Milestone 6**: Windows packaging, release optimization, overwrite safety.

---

## 📄 License

MIT License.
