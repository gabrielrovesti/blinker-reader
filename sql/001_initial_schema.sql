-- Blinker Reader Database Schema
-- Version: 0.1.0

-- Library items table
CREATE TABLE IF NOT EXISTS library_item (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    file_hash TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    title TEXT NOT NULL,
    author TEXT,
    publisher TEXT,
    subject TEXT,
    language TEXT,
    page_count INTEGER,
    created_at INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    indexed_at INTEGER NOT NULL
);

CREATE INDEX idx_library_item_hash ON library_item(file_hash);
CREATE INDEX idx_library_item_type ON library_item(file_type);
CREATE INDEX idx_library_item_title ON library_item(title);

-- Tags table
CREATE TABLE IF NOT EXISTS tag (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_tag_name ON tag(name);

-- Item-tag relationship (many-to-many)
CREATE TABLE IF NOT EXISTS item_tag (
    item_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (item_id, tag_id),
    FOREIGN KEY (item_id) REFERENCES library_item(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tag(id) ON DELETE CASCADE
);

-- Reading state table
CREATE TABLE IF NOT EXISTS reading_state (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL UNIQUE,
    current_page INTEGER NOT NULL DEFAULT 0,
    total_pages INTEGER NOT NULL DEFAULT 0,
    progress REAL NOT NULL DEFAULT 0.0,
    last_opened INTEGER NOT NULL,
    reading_time INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (item_id) REFERENCES library_item(id) ON DELETE CASCADE
);

CREATE INDEX idx_reading_state_item ON reading_state(item_id);
CREATE INDEX idx_reading_state_last_opened ON reading_state(last_opened);

-- Annotations table
CREATE TABLE IF NOT EXISTS annotation (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL,
    page INTEGER NOT NULL,
    range_x REAL NOT NULL,
    range_y REAL NOT NULL,
    range_width REAL NOT NULL,
    range_height REAL NOT NULL,
    kind TEXT NOT NULL,
    text TEXT NOT NULL,
    color TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    FOREIGN KEY (item_id) REFERENCES library_item(id) ON DELETE CASCADE
);

CREATE INDEX idx_annotation_item ON annotation(item_id);
CREATE INDEX idx_annotation_page ON annotation(item_id, page);
CREATE INDEX idx_annotation_kind ON annotation(kind);

-- Full-text search virtual table
CREATE VIRTUAL TABLE IF NOT EXISTS library_fts USING fts5(
    item_id UNINDEXED,
    title,
    author,
    subject,
    tags,
    content='library_item',
    content_rowid='rowid'
);

-- Triggers to keep FTS5 in sync
CREATE TRIGGER IF NOT EXISTS library_item_ai AFTER INSERT ON library_item BEGIN
    INSERT INTO library_fts(rowid, item_id, title, author, subject, tags)
    VALUES (NEW.rowid, NEW.id, NEW.title, NEW.author, NEW.subject, '');
END;

CREATE TRIGGER IF NOT EXISTS library_item_ad AFTER DELETE ON library_item BEGIN
    INSERT INTO library_fts(library_fts, rowid, item_id, title, author, subject, tags)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.title, OLD.author, OLD.subject, '');
END;

CREATE TRIGGER IF NOT EXISTS library_item_au AFTER UPDATE ON library_item BEGIN
    INSERT INTO library_fts(library_fts, rowid, item_id, title, author, subject, tags)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.title, OLD.author, OLD.subject, '');
    INSERT INTO library_fts(rowid, item_id, title, author, subject, tags)
    VALUES (NEW.rowid, NEW.id, NEW.title, NEW.author, NEW.subject, '');
END;

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
);

INSERT INTO schema_version (version, applied_at) VALUES (1, strftime('%s', 'now'));
