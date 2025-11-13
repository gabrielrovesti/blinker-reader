import { useParams } from "react-router-dom";
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "../styles/Reader.css";

interface ReaderSession {
  session_id: string;
  document_id: string;
  current_page: number;
  total_pages: number;
}

function Reader() {
  const { id } = useParams<{ id: string }>();
  const [session, setSession] = useState<ReaderSession | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [page, setPage] = useState(1);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    if (id) {
      openDocument(id);
    }
  }, [id]);

  const openDocument = async (documentId: string) => {
    try {
      const result = await invoke<ReaderSession>("open_document", {
        id: documentId,
      });
      setSession(result);
      setPage(1);
    } catch (error) {
      console.error("Failed to open document:", error);
    }
  };

  const handleSearch = async () => {
    if (!session) return;
    try {
      const results = await invoke("search_document", {
        session_id: session.session_id,
        query: searchQuery,
      });
      console.log("Search results:", results);
    } catch (error) {
      console.error("Search error:", error);
    }
  };

  const renderCurrentPage = async () => {
    if (!session) return;
    try {
      const result = await invoke<{ width: number; height: number; data: number[] }>(
        "render_page",
        { session_id: session.session_id, page }
      );
      const canvas = canvasRef.current;
      if (!canvas) return;
      canvas.width = result.width;
      canvas.height = result.height;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      const imageData = new ImageData(
        new Uint8ClampedArray(result.data),
        result.width,
        result.height
      );
      ctx.putImageData(imageData, 0, 0);
    } catch (error) {
      console.error("Render error:", error);
    }
  };

  useEffect(() => {
    if (session) {
      renderCurrentPage();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [session, page]);

  return (
    <div className="reader">
      <header className="reader-header">
        <button onClick={() => window.history.back()}>← Back</button>
        <div className="reader-controls">
          <input
            type="text"
            placeholder="Search in document..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
          />
          {session && (
            <span className="page-info">
              Page {session.current_page} / {session.total_pages}
            </span>
          )}
        </div>
      </header>

      <div className="reader-content">
        <aside className="reader-sidebar">
          <h3>Table of Contents</h3>
          <p>TODO: TOC navigation</p>
        </aside>

        <main className="reader-main">
          <div className="document-viewer">
            <canvas ref={canvasRef} />
            {session && (
              <div className="nav-controls">
                <button
                  onClick={() => setPage((p) => Math.max(1, p - 1))}
                  disabled={page <= 1}
                >
                  Prev
                </button>
                <span>
                  Page {page} / {session.total_pages}
                </span>
                <button
                  onClick={() => setPage((p) => Math.min(session.total_pages, p + 1))}
                  disabled={page >= session.total_pages}
                >
                  Next
                </button>
              </div>
            )}
          </div>
        </main>
      </div>
    </div>
  );
}

export default Reader;
