# Project Context: Rust Bulk Certificate Generator

## 1. Project Overview

Build a native desktop application named **Certificate Studio** using Rust and the `egui` / `eframe` GUI framework.

The application allows users to upload an existing certificate design as a background image, visually position and customize text elements on a canvas, import custom fonts, and automatically generate personalized certificates in bulk from CSV data.

The application must be a native Rust desktop application. Do not use React, Electron, Tauri, a web frontend, or a browser-based canvas. The entire GUI must be implemented using Rust with egui/eframe.

The primary target platform is Windows.

## 2. Core Requirements

### A. Certificate Canvas Editor

Users must be able to:

- Import a certificate background image in PNG or JPG format.
- Display the background image on a central design canvas.
- Preserve the image's aspect ratio.
- Support A4 landscape and portrait dimensions, along with custom page dimensions.
- Zoom in and out of the canvas.
- Add text elements using an Add Text button.
- Select individual text elements.
- Drag text elements to reposition them.
- Resize text boxes using selection handles.
- Edit text content.
- Change font family, font size, text color, and text alignment.
- Import custom TTF and OTF font files.
- Preview custom fonts in the canvas.
- Delete selected elements.
- Undo and redo changes.
- Save and reopen editable certificate projects.

The background image is not the editable certificate itself. It is a fixed design layer, and all personalized text must remain independently editable.

The canvas must support proper selection, hit testing, dragging, resizing, and coordinate conversion.

### B. Custom Font Management

Users must be able to import fonts from their computer.

Requirements:

- Support TTF and OTF files where compatible with the selected font libraries.
- Validate font files when importing them.
- Register imported fonts with egui for display in the editor.
- Allow individual text elements to use different fonts.
- Persist references to imported fonts in the saved project.
- Copy or package font assets into a managed project directory so that projects do not break when the original font file is moved.
- Use the appropriate font during final PDF and PNG rendering.
- Handle missing or invalid fonts gracefully.
- Prevent text from silently changing fonts during export.

Do not assume that registering a font in egui automatically makes it available to the PDF renderer. The editor and export engine need compatible, explicitly implemented font handling.

### C. Template and Project Persistence

A project contains:

- Project name and unique ID.
- Page dimensions.
- Background image asset reference.
- Positioned text elements.
- Text content and optional dynamic field keys.
- Font references and typography properties.
- Other element properties required by the editor.

Use Serde for serialization and JSON for the initial project format.

The project must be saved with its associated assets in a managed directory. Use stable asset references rather than fragile temporary paths.

Provide New Project, Open Project, Save Project, and Save As operations.

### D. CSV Import and Bulk Personalization

Users must be able to import CSV files containing participant information.

Example:

name,event,certificate_id
Aarav Patil,Code4Impact,CI-001
Meera Shah,Code4Impact,CI-002

The application must:

- Parse CSV headers and rows.
- Display a preview of imported data.
- Allow users to map CSV columns to text elements.
- Support dynamic placeholders such as `{{name}}`, `{{event}}`, and `{{certificate_id}}`.
- Preview a certificate for a selected participant.
- Detect missing values and invalid records.
- Provide a clear summary before batch generation.

Do not require users to manually edit JSON or source code to map fields.

### E. Export Engine

Implement three export options:

1. Individual PDF certificates.
2. Individual PNG certificates.
3. One combined PDF containing all generated certificates, one certificate per page.

Requirements:

- Preserve the original background image's proportions.
- Render text at the correct document coordinates.
- Preserve typography and alignment as closely as possible.
- Use consistent layout logic between the canvas preview and final exports.
- Sanitize generated filenames.
- Avoid overwriting existing files without user confirmation.
- Display batch progress and a final success/failure report.
- Handle large batches without freezing the UI.
- Use background processing where necessary.

The output must be suitable for printing. Provide configurable raster resolution for PNG output.

### F. QR Verification — Future Phase Only

Do not implement QR functionality in the first version.

However, design the architecture so that a QR code can later be introduced as another certificate element.

The future implementation may accept a URL or text, generate a QR code, and associate it with a certificate ID. A public verification website and validity database belong to a later phase.

Do not add QR dependencies or verification infrastructure to the initial milestone.

## 3. Required Technology Stack

Use Rust and the following libraries where appropriate:

- `eframe` and `egui` for the desktop interface.
- `rfd` for native file dialogs.
- `image` for image decoding and PNG rendering.
- `printpdf` for PDF generation.
- `fontdue` for font parsing and rasterization.
- `rustybuzz` for advanced text shaping when required.
- `csv` for CSV import.
- `serde` and `serde_json` for project persistence.
- `uuid` for stable element and project identifiers.
- `anyhow` and `thiserror` for error handling.
- `tracing` and `tracing-subscriber` for diagnostic logging.

Check the current compatible versions, Rust compiler requirements, and APIs before implementation. Resolve and pin dependency versions through Cargo. Do not blindly assume the proposed versions are mutually compatible.

Use Cargo as the build system. Keep `Cargo.lock` under version control for this application.

## 4. Proposed Project Structure

certificate-studio/
  Cargo.toml
  Cargo.lock
  assets/
  src/
    main.rs
    app.rs
    model/
      mod.rs
      project.rs
      element.rs
    editor/
      mod.rs
      canvas.rs
      interactions.rs
      properties.rs
    fonts/
      mod.rs
      manager.rs
    data/
      mod.rs
      csv_import.rs
      field_mapping.rs
    render/
      mod.rs
      layout.rs
    export/
      mod.rs
      png.rs
      pdf.rs
      batch.rs

You may add modules where justified, but maintain separation between the GUI, data model, font handling, rendering, persistence, and export systems.

Keep `main.rs` minimal. Do not place the entire application inside one source file.

## 5. Coordinate System and Rendering Architecture

This is a critical architectural requirement.

Store canvas positions and sizes in document units, preferably millimetres. Do not store positions in screen pixels.

The editor must convert document coordinates to screen coordinates using the canvas origin and current zoom.

The export engine must use the same document model and coordinate system.

The background image and text elements must render consistently in the editor and exported files.

Use a shared rendering/layout layer wherever practical. Do not create independent, inconsistent positioning logic for the canvas and export engine.

For an A4 landscape document, use 297 mm × 210 mm as the page dimensions.

Account for PDF coordinate conventions and unit conversions explicitly. Test the output using known reference positions.

## 6. GUI Layout

Create a professional, clean desktop interface.

### Top Toolbar
- New Project
- Open Project
- Save
- Undo
- Redo
- Zoom controls
- Preview
- Export

### Left Sidebar
- Import Background
- Add Text
- Imported Fonts
- Element list

### Center Panel
- Certificate canvas
- Background image
- Selectable text elements
- Resize handles
- Alignment guides
- Zoom and pan behavior

### Right Sidebar
- Selected element properties
- Text content
- Font family
- Font size
- Color
- Alignment
- Position and dimensions
- Dynamic field key

### Bulk Generation Panel
- Import CSV
- Preview records
- Map columns
- Select recipient
- Choose output formats
- Choose destination folder
- Generate certificates
- Display progress and errors

Use native egui widgets and custom painting where appropriate. Avoid unnecessary visual complexity.

## 7. Implementation Plan

Work incrementally. Do not attempt to implement every feature in a single code-generation pass.

### Milestone 1: Application Foundation

- Inspect the repository and existing source files before changing anything.
- Create or validate the Rust project.
- Resolve compatible dependencies.
- Launch a minimal eframe window.
- Establish the module structure.
- Implement a basic application layout.
- Ensure `cargo check` and `cargo run` succeed.

### Milestone 2: Functional Canvas

- Import a background image.
- Display it at the correct aspect ratio.
- Add text elements.
- Select, move, and resize text elements.
- Implement a properties panel.
- Support zoom and coordinate conversion.
- Add custom font loading and preview.

### Milestone 3: Save and Reopen Projects

- Define serializable project models.
- Save and reopen templates.
- Package or copy project assets.
- Preserve layout and font references.
- Validate that projects survive application restarts.

### Milestone 4: CSV and Dynamic Fields

- Import CSV data.
- Display record previews.
- Map fields to elements.
- Replace placeholders for each participant.
- Add validation for missing values.

### Milestone 5: Rendering and Export

- Implement a shared document rendering pipeline.
- Export individual PNGs.
- Export individual PDFs.
- Export a combined PDF.
- Test font embedding, text positioning, page dimensions, and image quality.
- Add batch progress and failure reporting.

### Milestone 6: Quality and Packaging

- Add undo/redo.
- Test long names and text overflow.
- Test missing assets and invalid files.
- Prevent accidental overwrites.
- Optimize large batches.
- Build and test the Windows release.

QR verification remains deferred.

## 8. Development Rules

Follow these rules throughout implementation:

1. Inspect existing files before creating or modifying them.
2. Do not overwrite working functionality without a clear reason.
3. Work in small, verifiable milestones.
4. Compile and test after each meaningful change.
5. Use the current documented APIs of the selected crate versions.
6. Do not invent library methods or rely on unverified examples.
7. Keep GUI code separate from business logic.
8. Avoid unnecessary dependencies.
9. Handle errors explicitly instead of silently ignoring failures.
10. Do not use placeholder implementations for features claimed to be complete.
11. Do not claim a feature works unless it has been implemented and tested.
12. Preserve existing user data and project files.
13. Explain important architectural decisions and any compromises.
14. When a build fails, inspect the actual compiler diagnostics and fix the underlying issue rather than repeatedly rewriting unrelated code.
15. Keep the application usable after every milestone.

## 9. Testing Requirements

At minimum, test:

- Importing a PNG and JPG background.
- Creating, moving, resizing, and deleting text elements.
- Switching between fonts and font sizes.
- Saving and reopening a project.
- Handling missing image and font assets.
- Importing CSV data with empty fields and long names.
- Correct placeholder replacement.
- Correct output filenames.
- PNG dimensions and resolution.
- PDF page dimensions and combined-PDF page count.
- Text position consistency between preview and export.
- Export failure reporting and file overwrite protection.

Add unit tests for document coordinate conversions, field resolution, filename sanitization, and project serialization.

## 10. Immediate Task

Start with Milestone 1 only.

First inspect the workspace and determine whether a Rust project already exists. If it does, preserve and work with it. If it does not, create the `certificate-studio` project.

Then:

1. Verify the Rust toolchain.
2. Configure compatible dependencies.
3. Create the minimal module structure.
4. Implement a working eframe window with a top toolbar, left sidebar, central canvas area, and right properties panel.
5. Add a functional native file picker for selecting a certificate background.
6. Display the selected image in the central panel.
7. Run `cargo check` and `cargo run` and fix any errors.

Do not implement bulk generation, PDF export, QR codes, or the complete editor during this first task.

At the end, report:
- Files created or modified.
- Dependencies added and resolved.
- Commands executed.
- Build and runtime test results.
- Known limitations.
- The next recommended milestone.

Begin by inspecting the workspace. Do not just return a plan; implement the first milestone in the actual project.
