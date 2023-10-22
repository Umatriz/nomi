import { createSignal } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";


function App() {
  return (
    <div>
        <select name="version">
            <option value="1.18">1.18</option>
            <option value="1.18.2">1.18.2</option>
        </select>
    </div>
  );
}

export default App;
