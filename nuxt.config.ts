import process from 'node:process'
import tailwindcss from '@tailwindcss/vite'

const host = process.env.TAURI_DEV_HOST

export default defineNuxtConfig({
  modules: ['@nuxt/eslint', '@nuxt/fonts', '@nuxt/icon/module', 'motion-v/nuxt', 'reka-ui/nuxt', '@vueuse/nuxt'],
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
  eslint: {
    config: { standalone: false }
  },
  typescript: {
    nodeTsConfig: { include: ['../bump.config.ts', '../eslint.config.ts', '../vitest.config.ts'] },
    tsConfig: { include: ['../test/**/*'] }
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
      ws: host
        ? {
            host,
            port: 3001
          }
        : undefined,
      watch: {
        ignored: ['**/src-tauri/**']
      }
    }
  },
  devServer: {
    host: host || '0.0.0.0'
  }
})
