# blinker-reader
A book/whatever reader simple (for real)

## PoC CLI

A minimal CLI is available to demonstrate the core library scan + database flow without the Tauri UI.

Build and run:

- Build workspace: `cargo build -p blinker-cli`
- Scan a folder into a SQLite DB: `cargo run -p blinker-cli -- scan <DIR> <DB_PATH>`

Example:

`cargo run -p blinker-cli -- scan C:\\Books .\\blinker.db`

This will:
- Initialize the SQLite schema (FTS5-enabled) if needed
- Recursively scan `<DIR>` for supported formats (pdf, epub, cbz, cbr, txt, md)
- Hash files with BLAKE3 and upsert entries into `library_item`

Notes:
- IDs are the file content hash (BLAKE3) for the PoC
- Tags, FTS queries and advanced metadata extraction are not implemented yet
