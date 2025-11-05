# Blinker Reader - Detailed Action Plan

**Status:** Scaffolding Complete (5% implemented) → MVP 0.1 Target
**Last Updated:** 2025-01-05
**Goal:** Transform stub implementations into a working document reader

---

## 📊 Current Status Assessment

### ✅ What's Working
- [x] Project structure and workspace configuration
- [x] Complete database schema (`sql/001_initial_schema.sql`)
- [x] Error handling system (`BlinkerError`)
- [x] Type definitions (Rust & TypeScript)
- [x] Tauri IPC commands registered (7 commands)
- [x] React frontend scaffolding with routing
- [x] CI/CD pipeline for 3 platforms
- [x] Documentation (architecture, specs)

### ❌ What Needs Implementation (Critical Path)
- [ ] Database initialization and migrations
- [ ] Library scanner with BLAKE3 hashing
- [ ] PDF rendering via PDFium
- [ ] Metadata extraction
- [ ] FTS5 search implementation
- [ ] IPC command implementations
- [ ] Frontend data flow
- [ ] EPUB, Comic, Text rendering
- [ ] Annotations storage
- [ ] Security hardening
- [ ] Comprehensive tests

---

## 🎯 Implementation Phases

## **PHASE 1: Database Foundation** (Est. 2-3 days)

The database is the backbone of the library system. Must be completed first.

### Task 1.1: Embed SQL Schema in Binary
**File:** `crates/blinker-core-library/src/database.rs`

**Actions:**
1. Add schema as embedded string constant or include_str!
   ```rust
   const SCHEMA_SQL: &str = include_str!("../../../sql/001_initial_schema.sql");
   ```
2. Create schema initialization function

**Dependencies:** None
**Test:** Schema string compiles and is accessible

---

### Task 1.2: Implement LibraryDatabase::new()
**File:** `crates/blinker-core-library/src/database.rs`

**Actions:**
1. Add `conn: Connection` field to `LibraryDatabase` struct
2. Implement initialization:
   ```rust
   pub fn new(db_path: &Path) -> Result<Self> {
       let conn = Connection::open(db_path)?;
       // Enable foreign keys
       conn.execute("PRAGMA foreign_keys = ON", [])?;
       // Run migrations
       Self::migrate(&conn)?;
       Ok(Self { conn })
   }
   ```
3. Add migration system:
   ```rust
   fn migrate(conn: &Connection) -> Result<()> {
       let version = Self::get_schema_version(conn)?;
       if version < 1 {
           conn.execute_batch(SCHEMA_SQL)?;
       }
       Ok(())
   }

   fn get_schema_version(conn: &Connection) -> Result<i32> {
       // Query schema_version table, default to 0
   }
   ```

**Dependencies:** Task 1.1
**Test:** Create database, verify tables exist, check foreign keys enabled

**Acceptance Criteria:**
- [x] Database file created at specified path
- [x] All 7 tables created (library_item, tag, item_tag, reading_state, annotation, library_fts, schema_version)
- [x] FTS5 triggers functional
- [x] Foreign keys enforced
- [x] Schema version = 1

---

### Task 1.3: Implement LibraryDatabase::add_or_update_path()
**File:** `crates/blinker-core-library/src/database.rs`

**Actions:**
1. Implement upsert logic using file_path as key
2. Update FTS5 table (triggers should auto-sync)
3. Handle deduplication via file_hash

**Pseudocode:**
```rust
pub fn add_or_update_path(&self, item: &LibraryItem) -> Result<i64> {
    // Check if hash exists (duplicate detection)
    if let Some(existing) = self.find_by_hash(&item.file_hash)? {
        // Update existing record
        return self.update_item(existing.id, item);
    }

    // Insert new item
    self.conn.execute(
        "INSERT INTO library_item (file_path, file_hash, file_type, ...)
         VALUES (?1, ?2, ?3, ...)
         ON CONFLICT(file_path) DO UPDATE SET ...",
        params![...],
    )?;

    Ok(self.conn.last_insert_rowid())
}
```

**Dependencies:** Task 1.2
**Test:** Insert item, verify in database, update same path, check dedup

---

### Task 1.4: Implement LibraryDatabase::get_item()
**File:** `crates/blinker-core-library/src/database.rs`

**Actions:**
1. Query by ID with JOIN to get tags
2. Return LibraryItem with all metadata

**SQL:**
```sql
SELECT l.*, GROUP_CONCAT(t.name) as tags
FROM library_item l
LEFT JOIN item_tag it ON l.id = it.item_id
LEFT JOIN tag t ON it.tag_id = t.id
WHERE l.id = ?1
GROUP BY l.id
```

**Dependencies:** Task 1.2
**Test:** Insert item with tags, retrieve by ID, verify all fields

---

### Task 1.5: Implement LibraryDatabase::query() with FTS5
**File:** `crates/blinker-core-library/src/database.rs`

**Actions:**
1. Build dynamic SQL from LibraryQuery filters
2. Use FTS5 for text search:
   ```sql
   SELECT item_id FROM library_fts WHERE library_fts MATCH ?
   ```
3. Support filters: type, tags, author, date ranges
4. Implement sorting (title, date, author)

**Dependencies:** Task 1.2
**Test:** Query by text, filter by type, combine filters

**Acceptance Criteria:**
- [x] Full-text search returns correct results
- [x] Filters work (type, tag, author)
- [x] Results properly sorted
- [x] Empty query returns all items

---

### Task 1.6: Add Database Unit Tests
**File:** `crates/blinker-core-library/tests/database_tests.rs`

**Test Cases:**
1. Database creation and schema initialization
2. Insert and retrieve item
3. Update existing item (upsert)
4. Duplicate detection by hash
5. FTS5 search accuracy
6. Tag management (add, remove, query)
7. Foreign key cascade deletes

**Dependencies:** Tasks 1.2-1.5
**Test:** `cargo test -p blinker-core-library` passes

---

## **PHASE 2: Library Scanner** (Est. 2-3 days)

Scan filesystem, hash files, extract basic metadata, populate database.

### Task 2.1: Implement MetadataExtractor::extract_basic()
**File:** `crates/blinker-core-library/src/metadata.rs`

**Actions:**
1. Get file size, modified time from filesystem
2. Detect DocumentFormat from extension
3. Set basic metadata (filename as title initially)

**Dependencies:** Phase 1 complete
**Test:** Extract metadata from real file

---

### Task 2.2: Implement LibraryScanner::scan_paths()
**File:** `crates/blinker-core-library/src/scanner.rs`

**Actions:**
1. Walk directory tree recursively
2. Filter by extensions: .pdf, .epub, .cbz, .cbr, .txt, .md
3. For each file:
   - Calculate BLAKE3 hash
   - Extract basic metadata
   - Call `store.add_or_update_path()`
4. Build ScanReport (files_found, added, updated, errors)

**Dependencies:** Tasks 2.1, Phase 1
**Test:** Scan test directory with 10 files, verify report

**Implementation Notes:**
- Use `walkdir` crate for directory traversal
- Skip hidden files/directories (starts with .)
- Handle permission errors gracefully
- Stream processing for large libraries

---

### Task 2.3: Add Scanner Unit Tests
**File:** `crates/blinker-core-library/tests/scanner_tests.rs`

**Test Cases:**
1. Scan directory with mixed file types
2. Detect duplicates (same hash)
3. Update modified files (hash changed)
4. Skip unsupported formats
5. Handle nested directories
6. Report error files correctly

**Dependencies:** Task 2.2
**Test:** `cargo test scanner` passes

---

## **PHASE 3: PDF Rendering** (Est. 3-4 days)

Most important format for MVP. Get PDF working end-to-end.

### Task 3.1: Add PDFium Dependency
**File:** `crates/blinker-core-render/Cargo.toml`

**Actions:**
1. Uncomment or add: `pdfium-render = "0.8"`
2. Research Windows/macOS/Linux PDFium binary requirements
3. Document build process for each platform

**Dependencies:** None
**Test:** `cargo build -p blinker-core-render` succeeds on your OS

---

### Task 3.2: Implement PdfRenderer::new()
**File:** `crates/blinker-core-render/src/pdf.rs`

**Actions:**
1. Initialize PDFium library (disable JavaScript):
   ```rust
   use pdfium_render::prelude::*;

   pub struct PdfRenderer {
       document: PdfDocument<'static>,
   }

   impl PdfRenderer {
       pub fn new(path: &Path) -> Result<Self> {
           let pdfium = Pdfium::new(
               Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                   .or_else(|_| Pdfium::bind_to_system_library())?
           );

           // Disable JavaScript (security requirement)
           let document = pdfium.load_pdf_from_file(path, None)?;
           // Verify JS is disabled

           Ok(Self { document })
       }
   }
   ```

**Dependencies:** Task 3.1
**Test:** Open sample PDF, verify no panics

---

### Task 3.3: Implement PdfRenderer::page_count()
**File:** `crates/blinker-core-render/src/pdf.rs`

**Actions:**
```rust
fn page_count(&self) -> Result<usize> {
    Ok(self.document.pages().len())
}
```

**Dependencies:** Task 3.2
**Test:** Open multi-page PDF, verify count matches

---

### Task 3.4: Implement PdfRenderer::render_page()
**File:** `crates/blinker-core-render/src/pdf.rs`

**Actions:**
1. Render page to bitmap at specified DPI (default 150)
2. Return RenderedPage with RGBA8 data
   ```rust
   fn render_page(&self, page_num: usize, dpi: u32) -> Result<RenderedPage> {
       let page = self.document.pages().get(page_num)?;
       let render_config = PdfRenderConfig::new()
           .set_target_width((page.width().value * dpi as f32 / 72.0) as i32)
           .set_target_height((page.height().value * dpi as f32 / 72.0) as i32);

       let bitmap = page.render_with_config(&render_config)?;

       Ok(RenderedPage {
           width: bitmap.width() as u32,
           height: bitmap.height() as u32,
           data: bitmap.as_rgba_bytes().to_vec(),
       })
   }
   ```

**Dependencies:** Task 3.3
**Test:** Render page, verify bitmap size, save to PNG for visual check

---

### Task 3.5: Implement PdfRenderer::search()
**File:** `crates/blinker-core-render/src/pdf.rs`

**Actions:**
1. Search all pages for text query
2. Return matches with page number and position
   ```rust
   fn search(&self, query: &str) -> Result<Vec<RenderSearchMatch>> {
       let mut matches = Vec::new();
       for (page_idx, page) in self.document.pages().iter().enumerate() {
           let text = page.text()?;
           // Find all occurrences of query in text
           // Get bounding boxes for matches
           // Add to matches vec
       }
       Ok(matches)
   }
   ```

**Dependencies:** Task 3.4
**Test:** Search for known text in PDF, verify matches

---

### Task 3.6: Implement PDF Metadata Extraction
**File:** `crates/blinker-core-library/src/metadata.rs`

**Actions:**
1. Add `extract_pdf()` method using PDFium
2. Extract: title, author, subject, page count, language
3. Fall back to filename if metadata missing

**Dependencies:** Phase 3 (PDFium integrated)
**Test:** Extract metadata from PDF with/without embedded metadata

---

### Task 3.7: Add PDF Rendering Tests
**File:** `crates/blinker-core-render/tests/pdf_tests.rs`

**Test Cases:**
1. Open valid PDF
2. Reject invalid/corrupted PDF
3. Count pages correctly
4. Render first page (smoke test)
5. Search returns expected matches
6. Metadata extraction accuracy

**Dependencies:** Tasks 3.2-3.6
**Test:** `cargo test pdf` passes

---

## **PHASE 4: IPC Integration** (Est. 1-2 days)

Connect Tauri commands to core functionality.

### Task 4.1: Create Tauri Application State
**File:** `apps/desktop/src-tauri/src/main.rs`

**Actions:**
1. Create AppState struct:
   ```rust
   use std::sync::{Arc, Mutex};
   use blinker_core_library::{LibraryDatabase, LibraryScanner};

   struct AppState {
       db: Arc<Mutex<LibraryDatabase>>,
       scanner: Arc<LibraryScanner>,
       sessions: Arc<Mutex<HashMap<String, Box<dyn DocumentRenderer>>>>,
   }
   ```
2. Initialize on startup:
   ```rust
   fn main() {
       let app_data_dir = /* get Tauri app data dir */;
       let db_path = app_data_dir.join("library.db");
       let db = LibraryDatabase::new(&db_path).expect("Failed to init database");

       let state = AppState {
           db: Arc::new(Mutex::new(db)),
           scanner: Arc::new(LibraryScanner::new()),
           sessions: Arc::new(Mutex::new(HashMap::new())),
       };

       tauri::Builder::default()
           .manage(state)
           .invoke_handler(/* ... */)
           .run(/* ... */);
   }
   ```

**Dependencies:** Phase 1, Phase 2
**Test:** App starts, database created in correct location

---

### Task 4.2: Implement scan_library Command
**File:** `apps/desktop/src-tauri/src/main.rs`

**Actions:**
```rust
#[tauri::command]
fn scan_library(
    paths: Vec<String>,
    state: State<AppState>
) -> Result<ScanReport, String> {
    let db = state.db.lock().unwrap();
    let scanner = &state.scanner;

    let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    let report = scanner.scan_paths(&paths, &*db)
        .map_err(|e| e.to_string())?;

    Ok(report)
}
```

**Dependencies:** Tasks 4.1, Phase 2
**Test:** Call from frontend, verify database populated

---

### Task 4.3: Implement query_library Command
**File:** `apps/desktop/src-tauri/src/main.rs`

**Actions:**
```rust
#[tauri::command]
fn query_library(
    query: LibraryQuery,
    state: State<AppState>
) -> Result<Vec<LibraryItem>, String> {
    let db = state.db.lock().unwrap();
    db.query(&query).map_err(|e| e.to_string())
}
```

**Dependencies:** Task 4.1, Phase 1
**Test:** Query from frontend, verify results

---

### Task 4.4: Implement open_document Command
**File:** `apps/desktop/src-tauri/src/main.rs`

**Actions:**
```rust
#[tauri::command]
fn open_document(
    id: i64,
    state: State<AppState>
) -> Result<ReaderSession, String> {
    let db = state.db.lock().unwrap();
    let item = db.get_item(id).map_err(|e| e.to_string())?;

    let renderer = AnyRenderer::open_for(&item.file_path)
        .map_err(|e| e.to_string())?;

    let session_id = Uuid::new_v4().to_string();
    let total_pages = renderer.page_count().unwrap_or(0);

    let mut sessions = state.sessions.lock().unwrap();
    sessions.insert(session_id.clone(), renderer);

    Ok(ReaderSession {
        session_id,
        item_id: id,
        total_pages,
        current_page: 0,
    })
}
```

**Dependencies:** Task 4.1, Phase 3
**Test:** Open PDF, verify session created

---

### Task 4.5: Implement search_document Command
**File:** `apps/desktop/src-tauri/src/main.rs`

**Actions:**
```rust
#[tauri::command]
fn search_document(
    session_id: String,
    query: String,
    state: State<AppState>
) -> Result<Vec<SearchMatch>, String> {
    let sessions = state.sessions.lock().unwrap();
    let renderer = sessions.get(&session_id)
        .ok_or("Session not found")?;

    let matches = renderer.search(&query)
        .map_err(|e| e.to_string())?;

    // Convert to SearchMatch format
    Ok(matches.into_iter().map(|m| SearchMatch {
        page: m.page,
        text: m.text,
        x: m.x,
        y: m.y,
    }).collect())
}
```

**Dependencies:** Task 4.4, Phase 3
**Test:** Search in open document, verify matches

---

### Task 4.6: Add render_page Command (New)
**File:** `apps/desktop/src-tauri/src/main.rs`

**Note:** This command is missing from the original plan but essential for rendering.

**Actions:**
```rust
#[tauri::command]
fn render_page(
    session_id: String,
    page_num: usize,
    dpi: Option<u32>,
    state: State<AppState>
) -> Result<RenderedPage, String> {
    let sessions = state.sessions.lock().unwrap();
    let renderer = sessions.get(&session_id)
        .ok_or("Session not found")?;

    renderer.render_page(page_num, dpi.unwrap_or(150))
        .map_err(|e| e.to_string())
}
```

**Dependencies:** Task 4.4, Phase 3
**Test:** Render page, display in frontend

---

## **PHASE 5: Frontend Integration** (Est. 2-3 days)

Connect UI to working backend.

### Task 5.1: Implement Library Scanning in Home.tsx
**File:** `apps/desktop/ui/src/pages/Home.tsx`

**Actions:**
1. Add folder picker dialog:
   ```typescript
   import { open } from '@tauri-apps/api/dialog';
   import { invoke } from '@tauri-apps/api/tauri';

   const handleScan = async () => {
       const selected = await open({
           directory: true,
           multiple: true,
       });

       if (selected) {
           const paths = Array.isArray(selected) ? selected : [selected];
           const report = await invoke<ScanReport>('scan_library', { paths });
           console.log('Scan complete:', report);
           // Refresh library
           await loadLibrary();
       }
   };
   ```

**Dependencies:** Phase 4
**Test:** Click scan, select folder, verify items appear

---

### Task 5.2: Implement Library Display in Home.tsx
**File:** `apps/desktop/ui/src/pages/Home.tsx`

**Actions:**
1. Load library on mount
2. Display in grid/list
3. Handle search input
   ```typescript
   const [library, setLibrary] = useState<LibraryItem[]>([]);
   const [searchQuery, setSearchQuery] = useState('');

   const loadLibrary = async () => {
       const query = {
           text: searchQuery || undefined,
           file_type: undefined,
           tags: [],
           sort_by: 'title',
           limit: 100,
           offset: 0,
       };
       const items = await invoke<LibraryItem[]>('query_library', { query });
       setLibrary(items);
   };

   useEffect(() => {
       loadLibrary();
   }, [searchQuery]);
   ```

**Dependencies:** Task 5.1
**Test:** Library displays items, search filters results

---

### Task 5.3: Implement Document Opening in Reader.tsx
**File:** `apps/desktop/ui/src/pages/Reader.tsx`

**Actions:**
1. Open document on mount (get ID from route params)
2. Fetch and display pages
   ```typescript
   const { id } = useParams();
   const [session, setSession] = useState<ReaderSession | null>(null);
   const [currentPage, setCurrentPage] = useState(0);
   const [pageImage, setPageImage] = useState<string>('');

   useEffect(() => {
       const openDoc = async () => {
           const sess = await invoke<ReaderSession>('open_document', {
               id: parseInt(id!)
           });
           setSession(sess);
       };
       openDoc();
   }, [id]);

   useEffect(() => {
       if (session) {
           renderPage(currentPage);
       }
   }, [session, currentPage]);

   const renderPage = async (pageNum: number) => {
       const rendered = await invoke<RenderedPage>('render_page', {
           sessionId: session!.session_id,
           pageNum,
       });

       // Convert RGBA8 buffer to base64 image
       const canvas = document.createElement('canvas');
       canvas.width = rendered.width;
       canvas.height = rendered.height;
       const ctx = canvas.getContext('2d')!;
       const imageData = new ImageData(
           new Uint8ClampedArray(rendered.data),
           rendered.width,
           rendered.height
       );
       ctx.putImageData(imageData, 0, 0);
       setPageImage(canvas.toDataURL());
   };
   ```

**Dependencies:** Phase 4
**Test:** Click document in library, opens in reader, displays page

---

### Task 5.4: Add Navigation Controls
**File:** `apps/desktop/ui/src/pages/Reader.tsx`

**Actions:**
1. Previous/next buttons
2. Page number input
3. Keyboard shortcuts (arrow keys)

**Dependencies:** Task 5.3
**Test:** Navigate between pages

---

### Task 5.5: Implement Search in Reader
**File:** `apps/desktop/ui/src/pages/Reader.tsx`

**Actions:**
1. Search input in header
2. Call search_document
3. Display matches, jump to page

**Dependencies:** Task 5.3
**Test:** Search for text, verify navigation to results

---

## **PHASE 6: Additional Formats** (Est. 3-4 days)

Implement EPUB, Comics, Text/Markdown rendering.

### Task 6.1: Implement EPUB Rendering
**Files:**
- `crates/blinker-core-render/Cargo.toml` - Add epub parser dependency
- `crates/blinker-core-render/src/epub.rs`

**Actions:**
1. Add dependency: `epub = "2.0"` or similar
2. Parse EPUB structure (OPF manifest, spine)
3. Extract HTML content
4. Sanitize HTML (remove scripts, block remote resources)
5. Render to pages (consider using webview or converting to images)

**Dependencies:** Phase 3 complete
**Test:** Open EPUB, display content, verify no scripts execute

---

### Task 6.2: Implement Comic Rendering (CBZ/CBR)
**Files:**
- `crates/blinker-core-render/Cargo.toml` - Add zip/unrar
- `crates/blinker-core-render/src/comic.rs`

**Actions:**
1. Add dependencies: `zip = "0.6"`, consider unrar support
2. Extract archive in-memory
3. Validate paths (prevent traversal attacks)
4. Sort images by filename
5. Decode images (JPEG, PNG, WebP)
6. Return as RenderedPage per image

**Dependencies:** Phase 3 complete
**Test:** Open CBZ, display images in order

---

### Task 6.3: Implement Text/Markdown Rendering
**Files:**
- `crates/blinker-core-render/Cargo.toml` - Add markdown parser
- `crates/blinker-core-render/src/text.rs`

**Actions:**
1. Add dependency: `pulldown-cmark = "0.9"` for Markdown
2. Plain text: render to pages (break by line count or char limit)
3. Markdown: parse to HTML, render
4. Simple pagination

**Dependencies:** Phase 3 complete
**Test:** Open TXT and MD files, verify rendering

---

## **PHASE 7: Annotations** (Est. 2 days)

Implement annotation storage and retrieval.

### Task 7.1: Implement AnnotationManager Storage
**File:** `crates/blinker-core-annot/src/lib.rs`

**Actions:**
1. Add SQLite connection (share from LibraryDatabase or separate)
2. Implement `add()`:
   ```rust
   fn add(&self, annotation: &Annotation) -> Result<i64> {
       self.conn.execute(
           "INSERT INTO annotation (item_id, page, range_x, range_y, range_width, range_height, kind, text, color)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
           params![...],
       )?;
       Ok(self.conn.last_insert_rowid())
   }
   ```

**Dependencies:** Phase 1
**Test:** Add annotation, verify in database

---

### Task 7.2: Implement Annotation Querying
**File:** `crates/blinker-core-annot/src/lib.rs`

**Actions:**
```rust
fn list(&self, item_id: i64) -> Result<Vec<Annotation>> {
    let mut stmt = self.conn.prepare(
        "SELECT * FROM annotation WHERE item_id = ?1 ORDER BY page, created_at"
    )?;
    let rows = stmt.query_map([item_id], |row| {
        Ok(Annotation { /* map fields */ })
    })?;
    rows.collect()
}
```

**Dependencies:** Task 7.1
**Test:** List annotations for item

---

### Task 7.3: Implement Export Functions
**File:** `crates/blinker-core-annot/src/lib.rs`

**Actions:**
1. JSON export: serialize Vec<Annotation> with serde_json
2. Markdown export: format as bulleted list with page numbers

**Dependencies:** Task 7.2
**Test:** Export annotations, verify format

---

### Task 7.4: Wire Annotation Commands
**File:** `apps/desktop/src-tauri/src/main.rs`

**Actions:**
1. Add AnnotationManager to AppState
2. Implement `add_annotation` and `list_annotations` commands

**Dependencies:** Phase 7 tasks
**Test:** Add/list annotations via IPC

---

## **PHASE 8: Security & Polish** (Est. 3-4 days)

Harden security, improve UX, prepare for release.

### Task 8.1: Implement HTML Sanitization
**File:** `crates/blinker-core-security/src/sanitizer.rs`

**Actions:**
1. Add dependency: `ammonia = "3.3"` (HTML sanitizer)
2. Implement `sanitize_html()`:
   ```rust
   use ammonia::Builder;

   pub fn sanitize_html(input: &str) -> String {
       Builder::default()
           .link_rel(None) // Remove all rel attributes
           .url_relative(ammonia::UrlRelative::Deny) // Block relative URLs
           .clean(input)
           .to_string()
   }
   ```

**Dependencies:** None
**Test:** Sanitize malicious HTML, verify scripts removed

---

### Task 8.2: Implement Path Validation
**File:** `crates/blinker-core-security/src/lib.rs`

**Actions:**
```rust
pub fn validate_archive_path(path: &str) -> Result<()> {
    if path.contains("..") || path.starts_with('/') || path.contains('\\') {
        return Err(BlinkerError::Security("Path traversal attempt".into()));
    }
    Ok(())
}
```

**Dependencies:** None
**Test:** Validate safe and unsafe paths

---

### Task 8.3: Apply Sanitization in EPUB Renderer
**File:** `crates/blinker-core-render/src/epub.rs`

**Actions:**
1. Import sanitize_html from blinker-core-security
2. Sanitize all HTML content before rendering

**Dependencies:** Task 8.1, Task 6.1
**Test:** Open EPUB with scripts, verify sanitized

---

### Task 8.4: Platform-Specific Sandboxing (Optional for MVP)
**Files:** `crates/blinker-core-security/src/lib.rs`

**Note:** This is complex and can be deferred to 0.4.

**Actions:**
1. Windows: Implement AppContainer (use winapi crate)
2. macOS: Configure App Sandbox entitlements
3. Linux: Implement seccomp-bpf filter

**Dependencies:** None
**Test:** Per-platform testing

---

### Task 8.5: UI Polish
**Files:** Various CSS and component files

**Actions:**
1. Add loading states (spinners)
2. Error handling and user feedback
3. Keyboard shortcuts documentation
4. Dark/light theme toggle
5. Accessibility improvements (ARIA labels)

**Dependencies:** Phase 5
**Test:** Manual UX testing

---

### Task 8.6: Performance Optimization
**Actions:**
1. Add page caching in renderers
2. Lazy load library items (pagination)
3. Background pre-rendering of next page
4. Database query optimization (indexes, prepared statements)

**Dependencies:** All phases
**Test:** Benchmark startup time, render time, memory usage

---

## **PHASE 9: Testing & Documentation** (Est. 2-3 days)

Comprehensive testing and documentation before release.

### Task 9.1: Integration Tests
**File:** `tests/integration_tests.rs` (workspace-level)

**Test Scenarios:**
1. End-to-end: Scan folder → Query library → Open PDF → Render page
2. Annotation round-trip: Add → List → Export
3. Search: Library search + in-document search
4. Deduplication: Add same file twice

**Dependencies:** All phases
**Test:** `cargo test --all` passes

---

### Task 9.2: Frontend Tests
**Files:** `apps/desktop/ui/src/**/*.test.tsx`

**Actions:**
1. Component tests (React Testing Library)
2. E2E tests (consider Playwright or Tauri's testing tools)

**Dependencies:** Phase 5
**Test:** `npm test` passes

---

### Task 9.3: Security Tests
**Actions:**
1. Test malicious PDFs (crafted files, fuzz testing)
2. Test EPUB with scripts
3. Test CBZ with path traversal
4. Verify no network calls (use network monitoring)

**Dependencies:** Phase 8
**Test:** Security audit passes

---

### Task 9.4: Performance Benchmarks
**File:** `benches/rendering.rs`

**Actions:**
1. Benchmark cold startup time
2. Benchmark PDF render time (target < 16ms)
3. Benchmark library indexing (1k files < 90s)
4. Memory profiling

**Dependencies:** All phases
**Test:** All benchmarks within budget

---

### Task 9.5: Update Documentation
**Files:** `README.md`, `docs/USER_GUIDE.md` (new)

**Actions:**
1. Update README with build instructions
2. Create user guide with screenshots
3. Document keyboard shortcuts
4. API documentation (cargo doc)

**Dependencies:** All phases
**Test:** Documentation builds, no broken links

---

## **PHASE 10: Packaging & Release** (Est. 1-2 days)

Build and distribute for all platforms.

### Task 10.1: Configure Tauri Bundler
**File:** `apps/desktop/src-tauri/tauri.conf.json`

**Actions:**
1. Set version, app ID, icons
2. Configure code signing (Windows Authenticode, macOS notarization)
3. Set bundle targets per platform

**Dependencies:** All phases complete
**Test:** Build bundles for 3 OS

---

### Task 10.2: CI/CD Release Pipeline
**File:** `.github/workflows/release.yml`

**Actions:**
1. Trigger on version tags (v0.1.0)
2. Build matrix for 3 OS
3. Run full test suite
4. Upload artifacts to GitHub Releases
5. Generate changelog

**Dependencies:** Task 10.1
**Test:** Manual release dry-run

---

### Task 10.3: First Release (v0.1.0)
**Actions:**
1. Tag repository: `git tag v0.1.0`
2. Push tag: `git push origin v0.1.0`
3. Monitor CI build
4. Download and test artifacts
5. Publish release notes

**Dependencies:** All tasks complete
**Test:** Manual installation on 3 OS

---

## 📋 Definition of Done (MVP 0.1)

### Functional Requirements
- [x] Open and read PDF files
- [x] Library indexing with scan functionality
- [x] Full-text search on metadata (title/author)
- [x] Basic UI with Home and Reader views
- [x] Portable builds for Windows/macOS/Linux

### Non-Functional Requirements
- [x] Cold startup < 700 ms (SATA disk)
- [x] Memory usage < 120 MB for library of ~1k documents
- [x] PDF render < 16 ms average (after warm-up)
- [x] Zero network calls (verified)
- [x] No JavaScript execution in PDFs

### Quality Gates
- [x] All unit tests pass (`cargo test --all`)
- [x] Integration tests pass
- [x] CI builds green on 3 platforms
- [x] Security audit clean (cargo-audit, cargo-deny)
- [x] No Clippy warnings
- [x] Code formatted (rustfmt)

---

## 🔄 Iteration Strategy

### Weekly Cycles
1. **Week 1:** Phase 1 + Phase 2 (Database + Scanner)
2. **Week 2:** Phase 3 + Phase 4 (PDF + IPC)
3. **Week 3:** Phase 5 (Frontend Integration)
4. **Week 4:** Phase 6-7 (Additional Formats + Annotations)
5. **Week 5:** Phase 8-9 (Security + Testing)
6. **Week 6:** Phase 10 (Packaging + Release)

### Daily Workflow
1. Pick next task from current phase
2. Implement feature
3. Write tests
4. Run `cargo test`, `cargo clippy`, `cargo fmt`
5. Commit with descriptive message
6. Update this document (check boxes)

### Risk Mitigation
- **PDFium integration fails:** Use pdf-rs or similar pure Rust library
- **EPUB rendering too complex:** Defer to v0.2, focus on PDF
- **Performance targets not met:** Profile and optimize, adjust targets if necessary
- **Cross-platform build issues:** Focus on one platform first (your dev OS)

---

## 📊 Progress Tracking

**Last Updated:** 2025-01-05
**Current Phase:** 1 (Database Foundation)
**Overall Completion:** 5%

### Phase Completion
- [ ] Phase 1: Database Foundation (0%)
- [ ] Phase 2: Library Scanner (0%)
- [ ] Phase 3: PDF Rendering (0%)
- [ ] Phase 4: IPC Integration (0%)
- [ ] Phase 5: Frontend Integration (0%)
- [ ] Phase 6: Additional Formats (0%)
- [ ] Phase 7: Annotations (0%)
- [ ] Phase 8: Security & Polish (0%)
- [ ] Phase 9: Testing & Documentation (0%)
- [ ] Phase 10: Packaging & Release (0%)

---

## 🎯 Next Immediate Actions

**Start Here:**

1. **Task 1.1:** Embed SQL schema in `database.rs` (15 min)
2. **Task 1.2:** Implement `LibraryDatabase::new()` (1-2 hours)
3. **Task 1.3:** Implement `add_or_update_path()` (2-3 hours)
4. **Task 1.6:** Write database tests (1-2 hours)

**Goal for Day 1:** Complete Phase 1 Tasks 1.1-1.4
**Goal for Week 1:** Complete Phase 1 + Phase 2

---

## 📚 Reference

- **Specification PDF:** `Specifiche Progetto Reader.pdf`
- **Architecture Doc:** `docs/ARCHITECTURE.md`
- **Database Schema:** `sql/001_initial_schema.sql`
- **Original Plan:** `docs/PLAN.md`

---

## ✅ Checklist Before Starting Each Phase

- [ ] Dependencies installed and compiling
- [ ] Test files created
- [ ] Documentation reviewed
- [ ] Previous phase complete and tested

---

**Remember:** Focus on getting PDF rendering working end-to-end first. Everything else can be added iteratively. The goal is a working MVP, not perfection.

Good luck! 🚀
