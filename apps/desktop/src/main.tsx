import React from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { InvitationWindow } from "./InvitationWindow";
import "./styles.css";

const element = document.getElementById("root");
if (!element) throw new Error("Seven Desk root element not found");

createRoot(element).render(
  <React.StrictMode>
    {new URLSearchParams(window.location.search).get("window") === "invitation" ? <InvitationWindow /> : <App />}
  </React.StrictMode>
);
