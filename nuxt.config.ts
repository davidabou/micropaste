import process from 'node:process'
import tailwindcss from '@tailwindcss/vite'

const host = process.env.TAURI_DEV_HOST

export default defineNuxtConfig({
  modules: ['@nuxt/fonts', '@nuxt/icon/module', 'motion-v/nuxt', 'reka-ui/nuxt', '@vueuse/nuxt'],
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  app: {
    head: {
      htmlAttrs: { lang: 'en' }
    }
  },
  experimental: {
    typedPages: true
  },
  css: ['@/assets/css/main.css'],
  typescript: {
    nodeTsConfig: { include: ['../bump.config.ts'] }
  },
  fonts: {
    defaults: {
      weights: ['400 700'],
      subsets: ['latin', 'latin-ext']
    },
    families: [
      { name: 'Segoe UI', provider: 'none' }
    ]
  },
  icon: {
    provider: 'none',
    clientBundle: {
      scan: true,
      sizeLimitKb: 256
    }
  },
  vite: {
    plugins: [tailwindcss()],
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_'],
    server: {
      strictPort: true,
      ws: host ? {
        host: host,
        port: 3001
      } : undefined,
      watch: {
        ignored: ['**/src-tauri/**']
      }
    }
  },
  devServer: {
    host: host || '0.0.0.0'
  }
})
