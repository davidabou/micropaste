import process from 'node:process'
import tailwindcss from '@tailwindcss/vite'

const host = process.env.TAURI_DEV_HOST

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: ['@nuxt/fonts', '@nuxt/icon/module', 'motion-v/nuxt', 'reka-ui/nuxt'],
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
  fonts: {
    defaults: {
      weights: ['400 700'],
      subsets: ['latin', 'latin-ext']
    },
    families: [
      // Windows system fonts: already installed, never fetch them from a provider
      { name: 'Segoe UI Variable Text', provider: 'none' },
      { name: 'Segoe UI', provider: 'none' }
    ]
  },
  icon: {
    // Desktop app served offline: never fetch icons from the Iconify API at runtime
    provider: 'none',
    clientBundle: {
      // Bundle every icon used literally in the source (e.g. <Icon name="gravity-ui:magnifier" />)
      scan: true,
      // Add dynamically built icon names here, e.g. 'gravity-ui:file'
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
