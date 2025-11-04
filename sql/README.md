# Database Migrations

This directory contains SQL migration scripts for the Blinker Reader database.

## Migration Files

Migrations are numbered sequentially and applied in order:

- `001_initial_schema.sql` - Initial database schema with FTS5

## Schema Overview

### Core Tables

- **library_item**: Document metadata and file information
- **tag**: User-defined tags
- **item_tag**: Many-to-many relationship between items and tags
- **reading_state**: Current reading progress per document
- **annotation**: User annotations (highlights, notes, bookmarks)

### Full-Text Search

- **library_fts**: FTS5 virtual table for fast metadata search

## Running Migrations

Migrations are automatically applied by the `blinker-core-library` crate on database initialization.
