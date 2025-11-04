import { Routes, Route } from "react-router-dom";
import Home from "./pages/Home";
import Reader from "./pages/Reader";
import "./styles/App.css";

function App() {
  return (
    <div className="app">
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/reader/:id" element={<Reader />} />
      </Routes>
    </div>
  );
}

export default App;
