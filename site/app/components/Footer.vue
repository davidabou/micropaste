<script setup lang="ts">
const columns = [
  {
    title: 'Menu',
    links: sections
  },
  {
    title: 'More projects',
    links: [
      { label: 'Harmonix', to: 'https://github.com/harmonix-js/harmonix' },
      { label: 'Pmdr', to: 'https://pmdr.abou.dev' }
    ]
  }
]
</script>

<template>
  <footer class="w-full mt-36 border-t border-black/6 bg-white/50">
    <div class="flex flex-col gap-16 max-w-5xl mx-auto px-6 pt-16">
      <div class="grid gap-10 sm:grid-cols-[2fr_1fr_1fr]">
        <div class="flex flex-col gap-4">
          <a href="/" class="flex items-center gap-2 font-medium w-fit" @click.prevent="scrollToTop">
            <Icon name="gravity-ui:cubes-3-overlap" size="18" />

            Micropaste
          </a>

          <div class="font-medium text-3xl">
            <p>Copy anything.</p>
            <p class="font-title">See it all again.</p>
          </div>

          <p class="max-w-xs text-sm text-neutral-500">
            Your clipboard, but you can actually see it. A visual clipboard library for Windows 11.
          </p>

          <Button :to="RELEASES_URL" icon="mdi:microsoft-windows-classic" size="sm">
            Download for Windows
          </Button>

          <p class="text-xs text-neutral-400">
            © 2026 Micropaste - MIT License
          </p>

          <p class="font-medium text-xs">
            Craft with 🦾 by
            <NuxtLink to="https://github.com/davidabou" target="_blank">
              David Abou
            </NuxtLink>
          </p>
        </div>

        <nav v-for="column in columns" :key="column.title" class="flex flex-col gap-3 text-sm" :aria-label="column.title">
          <h3 class="font-medium text-neutral-400">{{ column.title }}</h3>
          <ul class="flex flex-col gap-2">
            <li v-for="link in column.links" :key="link.label">
              <a :href="link.to" :target="link.to.startsWith('http') ? '_blank' : undefined" class="font-medium underline-offset-4 decoration-neutral-400 hover:underline">
                {{ link.label }}
              </a>
            </li>
          </ul>
        </nav>
      </div>

      <svg aria-hidden="true" viewBox="0 0 1000 177" class="w-full text-black/5 select-none">
        <text x="-15.7" y="155" fill="currentColor" font-size="217.5" class="font-semibold tracking-tighter">
          Micropaste
        </text>
      </svg>
    </div>
  </footer>
</template>
