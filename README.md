# blinker-reader
A book/whatever reader simple (for real)

## PoC CLI

A minimal CLI is available to demonstrate the core library scan + database flow without the Tauri UI.

Build and run:

- Build only the CLI: `cargo build -p blinker-cli`
- Scan a folder into a SQLite DB: `cargo run -p blinker-cli -- scan <DIR> <DB_PATH>`

Example:

`cargo run -p blinker-cli -- scan .\\tmp_books $env:TEMP\\blinker_cli.db`

What it does:
- Initializes the SQLite schema (FTS5-enabled) if needed
- Recursively scans `<DIR>` for supported formats (pdf, epub, cbz, cbr, txt, md)
- Hashes files with BLAKE3 and upserts entries into `library_item`

Notes:
- IDs are the file content hash (BLAKE3) for the PoC
- PDF/EPUB metadata extraction is currently behind optional features.
  - Default build disables `pdf-metadata` to avoid requiring PDFium at build/runtime.
  - To enable: `cargo build -p blinker-core-library --features pdf-metadata,epub-metadata`
- Tags, advanced FTS queries, and rich metadata extraction are WIP
