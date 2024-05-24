import React from "react"
import ReactDOM from "react-dom/client"
import App from "@/App"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryParamProvider } from "use-query-params"
import { ReactRouter6Adapter } from "use-query-params/adapters/react-router-6"
import { BrowserRouter } from "react-router-dom"

import "@/global.css"

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <BrowserRouter>
      <QueryParamProvider adapter={ReactRouter6Adapter}>
        <ThemeProvider defaultTheme={'dark'}>
          <App />
        </ThemeProvider>
      </QueryParamProvider>
    </BrowserRouter>
  </React.StrictMode>
)