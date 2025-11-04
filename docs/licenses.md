# Third-Party Licenses

This document tracks all third-party dependencies and their licenses used in Blinker Reader.

## Blinker Reader License

Blinker Reader itself is licensed under the **Apache License 2.0**.

## Core Dependencies

### Rust Crates

| Crate | Version | License | Usage |
|-------|---------|---------|-------|
| tokio | 1.x | MIT | Async runtime |
| serde | 1.x | MIT/Apache-2.0 | Serialization |
| rusqlite | 0.32.x | MIT | SQLite bindings |
| blake3 | 1.x | CC0-1.0 / Apache-2.0 | Hashing |
| tauri | 1.x | MIT/Apache-2.0 | Desktop framework |

### PDFium

- **License**: BSD 3-Clause
- **Source**: https://pdfium.googlesource.com/pdfium/
- **Usage**: PDF rendering with JavaScript disabled
- **NOTICE**: See PDFium LICENSE file

### Frontend Dependencies

| Package | Version | License | Usage |
|---------|---------|---------|-------|
| react | 18.x | MIT | UI framework |
| react-dom | 18.x | MIT | React DOM renderer |
| react-router-dom | 6.x | MIT | Routing |
| @tauri-apps/api | 1.x | MIT/Apache-2.0 | Tauri frontend API |
| vite | 5.x | MIT | Build tool |
| typescript | 5.x | Apache-2.0 | Type system |

## Full License Texts

Complete license texts for all dependencies can be found in their respective package manifests:

- Rust crates: Check `Cargo.toml` files or crates.io
- NPM packages: Check `package.json` files or npmjs.com

## Compliance

All dependencies have been reviewed for license compatibility with Apache-2.0.

- No GPL dependencies
- All licenses are permissive (MIT, Apache-2.0, BSD, CC0)
- No proprietary dependencies

## Updates

This file is updated with each new dependency addition. Last updated: 2025-01-04
