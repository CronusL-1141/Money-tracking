import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { ThemeProvider } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";

import App from "./App";
import { ThemeContextProvider } from "./contexts/ThemeContext";
import { LanguageContextProvider } from "./contexts/LanguageContext";
import { NotificationProvider } from "./contexts/NotificationContext";

import "./styles/index.css";
import "./locales/i18n";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <LanguageContextProvider>
        <ThemeContextProvider>
          <NotificationProvider>
            <CssBaseline />
            <App />
          </NotificationProvider>
        </ThemeContextProvider>
      </LanguageContextProvider>
    </BrowserRouter>
  </React.StrictMode>,
);