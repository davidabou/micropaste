import tailwindcss from '@tailwindcss/vite'

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: ['@nuxt/fonts', '@nuxt/icon', '@vercel/analytics', 'motion-v/nuxt'],
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  app: {
    head: {
      htmlAttrs: { lang: 'en' },
      link: [
        { rel: 'icon', href: '/favicon.ico', sizes: 'any' },
        { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' }
      ],
      meta: [
        { name: 'theme-color', content: '#f5f5f5' }
      ]
    }
  },
  css: ['@/assets/css/main.css'],
  vite: {
    plugins: [
      tailwindcss()
    ]
  },
  fonts: {
    defaults: {
      weights: ['400 700']
    }
  },
  icon: {
    mode: 'svg'
  }
})
