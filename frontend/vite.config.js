import { defineConfig } from 'vite'

export default defineConfig(({ mode }) => {
  return {
    base: './',
    server: {
      port: 5173
    },
    build: {
      outDir: '../web',
      emptyOutDir: true
    },
    define: {
      '__APP_VERSION__': JSON.stringify(process.env.VITE_VERSION || 'dev')
    }
  }
})
