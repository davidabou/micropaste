<script setup lang="ts">
const scrolled = ref(false)

function onScroll() {
  scrolled.value = window.scrollY > 0
}

onMounted(() => {
  onScroll()
  window.addEventListener('scroll', onScroll, { passive: true })
})

onBeforeUnmount(() => window.removeEventListener('scroll', onScroll))
</script>

<template>
  <!-- Margins in px: Motion can't interpolate between 100% and calc(100% + 26px) -->
  <Motion
    as="nav"
    class="relative flex-1 p-3.25"
    :initial="false"
    :animate="{ marginLeft: scrolled ? 0 : -13, marginRight: scrolled ? 0 : -13 }"
    :transition="{ type: 'spring', bounce: 0.4, visualDuration: 0.35 }"
  >
    <!-- Glass on its own layer, faded with opacity: transitioning the shadow itself makes it pulse -->
    <Motion
      aria-hidden="true"
      class="absolute inset-0 rounded-2xl border bg-white/60 border-white/80 backdrop-blur-xl backdrop-saturate-170 shadow-[inset_0_1px_0_rgba(255,255,255,0.7),0_24px_60px_-28px_rgba(24,38,84,0.4)]"
      :initial="false"
      :animate="{ opacity: scrolled ? 1 : 0 }"
      :transition="{ type: 'spring', bounce: 0, visualDuration: 0.3 }"
    />

    <div class="relative flex items-center justify-between">
      <a href="/" class="flex items-center gap-2 font-medium px-2" @click.prevent="scrollToTop">
        <Icon name="gravity-ui:cubes-3-overlap" size="18" />

        Micropaste
      </a>

      <ul class="flex items-center gap-6 text-sm">
        <li v-for="section in sections" :key="section.to">
          <a :href="section.to" class="font-medium underline-offset-4 decoration-neutral-400 hover:underline">
            {{ section.label }}
          </a>
        </li>
      </ul>

      <ul class="flex items-center gap-3 text-sm">
        <li>
          <Button :to="REPO_URL" icon="octicon:mark-github-16" :icon-size="14" variant="ghost" size="sm">
            GitHub
          </Button>
        </li>
        <li>
          <Button :to="RELEASES_URL" icon="mdi:microsoft-windows-classic" size="sm">
            Download
          </Button>
        </li>
      </ul>
    </div>
  </Motion>
</template>
