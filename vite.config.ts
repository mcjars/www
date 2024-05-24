import path from "path"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'lib'
  }, resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
})