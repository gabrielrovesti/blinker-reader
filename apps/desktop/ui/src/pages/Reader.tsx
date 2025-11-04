import { useParams } from "react-router-dom";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "../styles/Reader.css";

interface ReaderSession {
  id: string;
  document_id: string;
  current_page: number;
  total_pages: number;
}

function Reader() {
  const { id } = useParams<{ id: string }>();
  const [session, setSession] = useState<ReaderSession | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

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
    } catch (error) {
      console.error("Failed to open document:", error);
    }
  };

  const handleSearch = async () => {
    if (!session) return;
    try {
      const results = await invoke("search_document", {
        sessionId: session.id,
        query: searchQuery,
      });
      console.log("Search results:", results);
    } catch (error) {
      console.error("Search error:", error);
    }
  };

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
            <p>Document viewer placeholder</p>
            <p>TODO: Render document pages</p>
          </div>
        </main>
      </div>
    </div>
  );
}

export default Reader;
