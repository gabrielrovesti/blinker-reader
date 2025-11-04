# Blinker Reader Architecture

## Overview

Blinker Reader follows a layered architecture with clear separation between rendering, library management, annotations, and security.

## High-Level Architecture

```
┌─────────────────────────────────────┐
│         UI Layer (React/TS)         │
│  - Home / Library View              │
│  - Reader View                      │
│  - Settings & Preferences           │
└─────────────┬───────────────────────┘
              │ Tauri IPC
┌─────────────▼───────────────────────┐
│      Tauri Backend (Rust)           │
│  - Command handlers                 │
│  - State management                 │
│  - Event system                     │
└─────────────┬───────────────────────┘
              │
    ┌─────────┴─────────┐
    │                   │
┌───▼────────┐  ┌──────▼──────────┐
│  Library   │  │    Renderer     │
│ Management │  │  - PDF          │
│  - Scanner │  │  - EPUB         │
│  - DB      │  │  - Comics       │
│  - Search  │  │  - Text/MD      │
└───┬────────┘  └──────┬──────────┘
    │                  │
┌───▼──────────────────▼──────────┐
│      Security Layer             │
│  - Sandboxing                   │
│  - Content sanitization         │
│  - Path validation              │
└─────────────────────────────────┘
```

## Core Crates

### blinker-core-common

Shared types, error definitions, and utilities.

**Exports:**
- `BlinkerError` - unified error type
- `DocumentFormat` - supported formats enum
- `Metadata` - document metadata struct

### blinker-core-library

Library indexing and management.

**Responsibilities:**
- Scan filesystem for documents
- Extract and store metadata
- Manage tags
- Full-text search with SQLite FTS5
- Track reading state

**Key Components:**
- `LibraryScanner` - walks directories, hashes files
- `LibraryDatabase` - SQLite operations
- `MetadataExtractor` - format-specific metadata parsing

### blinker-core-render

Document rendering for all supported formats.

**Responsibilities:**
- PDF rendering via PDFium (JS disabled)
- EPUB HTML/CSS flow layout
- Comic archive (CBZ/CBR) image extraction
- Text and Markdown rendering

**Key Components:**
- `PdfRenderer` - PDFium wrapper
- `EpubRenderer` - EPUB parser + sanitizer
- `ComicRenderer` - safe archive extraction
- `TextRenderer` - plain text and Markdown

### blinker-core-annot

Annotation management (highlights, notes, bookmarks).

**Responsibilities:**
- Create and store annotations
- Query annotations per document
- Export to JSON/Markdown
- Future: write-back to PDF annotations

### blinker-core-security

Security layer for sandboxing and sanitization.

**Responsibilities:**
- OS-level sandboxing
  - Windows: AppContainer
  - macOS: App Sandbox
  - Linux: seccomp-bpf
- HTML content sanitization for EPUB
- Path validation for archives
- Network blocking

## Data Flow

### Opening a Document

1. User clicks document in library
2. Frontend sends `open_document` command via Tauri IPC
3. Backend queries database for file path
4. Security layer validates path
5. Renderer loads document based on format
6. Session created with current state
7. Frontend receives rendered content

### Library Scanning

1. User triggers scan via UI
2. `LibraryScanner` walks directory tree
3. For each supported file:
   - Calculate BLAKE3 hash
   - Check for duplicates
   - Extract metadata
   - Insert/update database
4. Return scan report to UI

### Full-Text Search

1. User enters search query
2. Query sent to `LibraryDatabase`
3. SQLite FTS5 searches indexed metadata
4. Results ranked and returned
5. UI displays matching documents

## Security Model

### Threat Mitigation

| Threat | Mitigation |
|--------|-----------|
| Malicious PDFs | PDFium with JS disabled, no embedded file extraction |
| EPUB scripts | Content sanitization, remote resources blocked |
| Path traversal | Strict path validation, in-memory archive extraction |
| Code execution | WASM-only plugins (post-MVP), no Python/Java |
| Supply chain | cargo-audit, cargo-deny, locked dependencies |
| Network leaks | No network access by default, OS-level blocking |

### Sandboxing

Platform-specific sandboxing applied at startup:

- **Windows**: AppContainer with minimal capabilities
- **macOS**: App Sandbox with read-only document access
- **Linux**: seccomp-bpf filtering system calls

## Database Schema

See [sql/001_initial_schema.sql](../sql/001_initial_schema.sql) for full schema.

**Core Tables:**
- `library_item` - documents with metadata
- `tag` - user tags
- `item_tag` - many-to-many relationship
- `reading_state` - progress per document
- `annotation` - user annotations
- `library_fts` - FTS5 search index

## Performance Considerations

### Startup Time

Target: < 300ms on NVMe, < 700ms on SATA

Optimizations:
- Minimal dependencies
- Lazy loading of renderers
- No heavy initialization on startup
- SQLite with memory-mapped I/O

### Memory Usage

Target: < 120MB for library of ~1k documents

Optimizations:
- Rust's zero-cost abstractions
- No GC overhead
- Efficient SQLite queries
- On-demand page rendering

### Rendering Performance

Target: < 16ms per page (60fps)

Optimizations:
- GPU-accelerated rendering
- Page caching
- Progressive loading for large documents
- Background pre-rendering

## Future Enhancements

### Plugin System (v1.0+)

- WebAssembly-based plugins
- Capability-based security model
- TOML manifest with permissions
- Sandboxed API access

### Cloud Sync (v1.x+)

- Optional end-to-end encrypted sync
- Reading state synchronization
- Annotation sync
- No document content upload

## References

- [Technical Specification PDF](../Specifiche%20Progetto%20Reader.pdf)
- [Repository Structure](../README.md#project-structure)
- [IPC API Overview](./IPC_API.md) (TODO)
