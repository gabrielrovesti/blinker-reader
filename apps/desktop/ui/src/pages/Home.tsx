import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import { useNavigate } from "react-router-dom";
import "../styles/Home.css";

interface LibraryItem {
  id: string;
  path: string;
  title: string;
  author?: string;
  file_type: string;
  hash: string;
  tags: string[];
}

function Home() {
  const [library, setLibrary] = useState<LibraryItem[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const navigate = useNavigate();

  const handleScan = async () => {
    try {
      const selection = await open({ directory: true, multiple: true });
      const paths = Array.isArray(selection)
        ? (selection as string[])
        : selection
        ? [selection as string]
        : [];
      const result = await invoke("scan_library", { paths });
      console.log("Scan result:", result);
    } catch (error) {
      console.error("Scan error:", error);
    }
  };

  const handleSearch = async () => {
    try {
      const results = await invoke<LibraryItem[]>("query_library", {
        filters: { text: searchQuery, limit: 200 },
      });
      setLibrary(results);
    } catch (error) {
      console.error("Search error:", error);
    }
  };

  return (
    <div className="home">
      <header className="home-header">
        <h1>Blinker Reader</h1>
        <p>Blink. Open. Read.</p>
      </header>

      <div className="search-bar">
        <input
          type="text"
          placeholder="Search library..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSearch()}
        />
        <button onClick={handleSearch}>Search</button>
        <button onClick={handleScan}>Scan Library</button>
      </div>

      <div className="library-grid">
        {library.length === 0 ? (
          <div className="empty-state">
            <p>No documents in library</p>
            <p>Click "Scan Library" to get started</p>
          </div>
        ) : (
          library.map((item) => (
            <div
              key={item.id}
              className="library-item"
              onClick={() => navigate(`/reader/${encodeURIComponent(item.id)}`)}
              role="button"
              tabIndex={0}
              onKeyDown={(e) => e.key === "Enter" && navigate(`/reader/${encodeURIComponent(item.id)}`)}
            >
              <div className="item-cover">
                <span>{item.file_type.toUpperCase()}</span>
              </div>
              <div className="item-info">
                <h3>{item.title}</h3>
                {item.author && <p>{item.author}</p>}
                <div className="item-tags">
                  {item.tags.map((tag) => (
                    <span key={tag} className="tag">
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

export default Home;
