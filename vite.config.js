import { defineConfig } from 'vite'
import { readFileSync } from 'fs'
import { resolve } from 'path'

function getCargoVersion() {
  try {
    const cargoPath = resolve(__dirname, 'src-tauri/Cargo.toml')
    const cargoContent = readFileSync(cargoPath, 'utf-8')
    const match = cargoContent.match(/^version = "([^"]+)"/m)
    return match ? match[1] : 'dev'
  } catch {
    return 'dev'
  }
}

export default defineConfig(({ mode }) => {
  return {
    base: './',
    root: 'frontend',
    server: {
      port: 5173
    },
    build: {
      outDir: '../dist',
      emptyOutDir: true
    },
    define: {
      '__APP_VERSION__': JSON.stringify(process.env.VITE_VERSION || getCargoVersion())
    }
  }
})
