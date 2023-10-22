/* @refresh reload */
import { render } from "solid-js/web";

// import "./styles.css";
import '@radix-ui/themes/styles.css';
import App from "./App";

render(() => (<App />), document.getElementById("root") as HTMLElement);
