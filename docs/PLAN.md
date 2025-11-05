# Blinker Reader — Piano di Progetto e Cambiamenti

## Piano di Progetto (consolidato dalle specifiche)

Obiettivo
- Lettore documenti veloce, sicuro, minimale e locale, senza telemetria o lock‑in.

Ambito MVP (v0.1)
- Formati: PDF, EPUB, CBZ/CBR, TXT, Markdown.
- Libreria locale con metadati, tag e ricerca full‑text (titolo/autore/tag) + ricerca in‑documento.
- UI minimale, keyboard‑first, temi chiaro/scuro.
- Build portabili per Windows/macOS/Linux.
- Zero telemetria, nessuna connessione in uscita di default.

Non ambito MVP
- Conversioni, store/cataloghi, DRM, cloud sync (post 1.x).

Piattaforme e target prestazionali
- Windows 10+ x64, macOS 13+ (Intel/Apple), Linux x64 (glibc).
- Startup a freddo: < 300 ms (NVMe) / < 700 ms (SATA).
- Memoria a riposo: < 120 MB con ~1k documenti.
- Render pagina PDF 1080p: < 16 ms medio post warm‑up.
- Indicizzazione 1k elementi: < 90 s (incrementale).

Architettura
- UI: Tauri + WebView (React/TypeScript).
- Backend Tauri (Rust) + IPC.
- Core Rust: rendering (PDFium/EPUB/CBZ/TXT/MD), libreria/DB/FTS, annotazioni, sicurezza.
- DB: SQLite + FTS5.

Sicurezza (modello e mitigazioni)
- PDFium con JS disabilitato; blocco azioni rischiose.
- Sanitizzazione HTML EPUB; blocco risorse remote.
- CBZ/CBR: estrazione in‑memory con normalizzazione dei path.
- Estensioni WASM/WASI (post‑MVP), niente exec esterno.
- Sandboxing OS: AppContainer (Win), App Sandbox (macOS), seccomp‑bpf (Linux).
- Nessuna rete di default; supply chain audit (cargo‑audit/deny, lockfile).

Persistenza (schema principale)
- Tabelle: library_item, tag, item_tag, reading_state, annotation, virtual table FTS5 + trigger di sync; tracking schema_version.

Requisiti funzionali
- Apertura/lettura di PDF/EPUB/CBZ/CBR/TXT/Markdown; navigazione, progress bar, ricerca inline.
- Libreria: scansione percorsi, hash BLAKE3, estrazione metadati, dedup, FTS, filtri/ordinamenti base.
- Annotazioni: evidenziazioni, note, segnalibri; elenco ed export JSON/Markdown.
- UI: Home (search/scan), Dettaglio, Reader (TOC opz., ricerca, palette comandi).

Requisiti non funzionali
- Performance, sicurezza by design, UX accessibile e coerente, offline‑first/file‑system‑first, licenze tracciate.

API IPC (principali)
- library.scan(paths[]) -> ScanReport
- library.query(filters) -> LibraryItem[]
- library.updateMeta(id, fields) -> void
- reader.open(id) -> Session
- reader.search(sessionId, query) -> Matches[]
- annot.add(itemId, range, kind, text, color) -> Annotation
- annot.list(itemId) -> Annotation[]

Roadmap (alto livello)
- 0.1: base reader PDF, UI base, indice libreria, packaging.
- 0.2: EPUB + migliorie libreria (editing metadati, file watcher).
- 0.3: Annotazioni + theming.
- 0.4: Polish & Security (sandbox, a11y, i18n, fuzzing).
- 1.0: Plugin WASM, notarizzazione, crash reporter.

Test & CI
- Unit/integration; snapshot rendering PDF; fuzzing su formati; gate performance; audit sicurezza; build matrice 3 OS.

Prossimi passi (MVP)
- Integrare SQLite in LibraryDatabase e migrazioni dallo schema SQL.
- Implementare LibraryScanner (hashing BLAKE3, filtri estensioni, upsert + FTS).
- Integrare PDFium e validare il render della prima pagina.
- Collegare comandi IPC al core e popolare la UI.
- Aggiungere test per DB/metadata e smoke‑test render PDF.
- Pipeline CI minima (build 3 OS + audit).

## Cambiamenti effettuati (scaffold base)

Rendering
- Aggiunto trait `DocumentRenderer` con operazioni comuni (open, page_count, render_page, search).
- Aggiunti tipi `RenderedPage` (bitmap RGBA8) e `RenderSearchMatch`.
- Factory `AnyRenderer::open_for(path)` per selezione renderer da estensione.
- Implementazioni stub del trait per ogni renderer:
  - PdfRenderer, EpubRenderer, ComicRenderer, TextRenderer (metodi con TODO e ritorni placeholder).

Libreria
- Struttura `LibraryItem` (id, path, hash, type, size, metadata, tags).
- `LibraryQuery` per parametri di ricerca.
- Trait `LibraryStore` (add_or_update_path, get_item, query) e impl stub per `LibraryDatabase`.
- `LibraryScanner::scan_paths` ora accetta un generico `LibraryStore` (decoupling dal DB concreto).

Annotazioni
- Trait `AnnotationStore` (add, list, export_json, export_markdown) + adapter su `AnnotationManager`.

Sicurezza
- Trait `Sandboxer` con `apply()` + adapter per `Sandbox`.

UI TypeScript
- File `apps/desktop/ui/src/types.ts` con interfacce condivise per IPC e stato: `LibraryItem`, `ScanReport`, `ReaderSession`, `SearchMatch`, `Annotation`.

## Riferimenti ai file modificati/aggiunti
- crates/blinker-core-render/src/lib.rs — trait, factory e tipi di rendering.
- crates/blinker-core-render/src/pdf.rs — impl trait per PDF.
- crates/blinker-core-render/src/epub.rs — impl trait per EPUB.
- crates/blinker-core-render/src/comic.rs — impl trait per Comics.
- crates/blinker-core-render/src/text.rs — impl trait per Text/Markdown.
- crates/blinker-core-library/src/lib.rs — tipi e interfacce libreria.
- crates/blinker-core-library/src/database.rs — impl `LibraryStore` stub.
- crates/blinker-core-library/src/scanner.rs — scanner parametrico su `LibraryStore`.
- crates/blinker-core-annot/src/lib.rs — trait `AnnotationStore` + adapter.
- crates/blinker-core-security/src/lib.rs — trait `Sandboxer` + adapter.
- apps/desktop/ui/src/types.ts — interfacce TS condivise.

