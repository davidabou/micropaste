<script setup lang="ts">
const title = 'Micropaste - Visual clipboard manager for Windows 11'
const description = 'Micropaste keeps everything you copy on Windows 11 and shows it as rich cards: links, images, colors, code and files. Free, open source and local-first.'
const image = `${SITE_URL}/og-image.png`

useSeoMeta({
  title,
  description,
  ogType: 'website',
  ogSiteName: 'Micropaste',
  ogUrl: `${SITE_URL}/`,
  ogTitle: title,
  ogDescription: description,
  ogImage: image,
  ogImageWidth: 1200,
  ogImageHeight: 630,
  ogImageAlt: 'Micropaste: copy anything, see it all again',
  twitterCard: 'summary_large_image',
  twitterTitle: title,
  twitterDescription: description,
  twitterImage: image
})

useHead({
  link: [{ rel: 'canonical', href: `${SITE_URL}/` }],
  script: [{
    type: 'application/ld+json',
    innerHTML: JSON.stringify({
      '@context': 'https://schema.org',
      '@type': 'SoftwareApplication',
      'name': 'Micropaste',
      description,
      'url': `${SITE_URL}/`,
      image,
      'applicationCategory': 'UtilitiesApplication',
      'operatingSystem': 'Windows 11',
      'downloadUrl': RELEASES_URL,
      'license': 'https://opensource.org/licenses/MIT',
      'isAccessibleForFree': true,
      'offers': { '@type': 'Offer', 'price': 0, 'priceCurrency': 'USD' },
      'author': { '@type': 'Person', 'name': 'David Abou', 'url': 'https://github.com/davidabou' },
      'sameAs': [REPO_URL]
    })
  }]
})

const collectionFacts = [
  {
    title: 'One clip, many places',
    description: 'A clip can live in as many collections as you like, without ever leaving your history.'
  },
  {
    title: 'Drop from any app',
    description: 'Drag files, text or images from Explorer or your browser straight into a collection.'
  },
  {
    title: 'Kept for good',
    description: 'History tidies itself after 30 days by default. Clips in a collection are never cleaned up.'
  },
  {
    title: 'In your order',
    description: 'Drag tabs to reorder your collections, and cards to reorder what\'s inside.'
  }
]

const windowsFacts = [
  {
    icon: 'gravity-ui:layers',
    title: 'Mica, light or dark',
    description: 'The window wears Windows 11\'s Mica material and follows your system theme.'
  },
  {
    icon: 'gravity-ui:keyboard',
    title: 'One shortcut away',
    description: 'Alt+Shift+V opens it in the bottom right corner, from any app. Pick your own in the settings.'
  },
  {
    icon: 'gravity-ui:tray',
    title: 'Lives in the tray',
    description: 'Closing it only hides it. It starts with Windows, quietly, so nothing you copy is missed.'
  },
  {
    icon: 'gravity-ui:folder',
    title: 'Speaks Explorer',
    description: 'File thumbnails, type names and app icons come straight from Windows, and drags land in any app.'
  }
]
</script>

<template>
  <div class="flex flex-col">
    <Motion class="sticky top-0 z-50 flex justify-center w-full max-w-5xl mx-auto p-6" v-bind="appear(1.8, -20)">
      <Navbar />
    </Motion>

    <main class="flex flex-col w-full max-w-5xl mx-auto">
      <section class="flex flex-col items-center gap-12 w-full py-36">
        <h1 class="font-medium text-center text-8xl tracking-tighter">
          <TextAnimate text="Copy anything." />
          <br>
          <TextAnimate text="See it all again." class="font-title" :delay="0.5" />
        </h1>

        <Motion as="p" class="max-w-2xl mx-auto text-center text-lg text-neutral-600" v-bind="appear(1)">
          Micropaste keeps everything you copy, previews it as a rich card, and hands it back in one click or drag. Your clipboard, but you can actually see it.
        </Motion>

        <Motion class="flex justify-center gap-4" v-bind="appear(1.4)">
          <Button :to="RELEASES_URL" icon="mdi:microsoft-windows-classic">
            Download for Windows
          </Button>

          <Button :to="REPO_URL" icon="octicon:mark-github-16" :icon-size="16" variant="ghost">
            View on GitHub
          </Button>
        </Motion>

        <Motion as="small" class="flex items-center gap-3 text-neutral-400" v-bind="appear(1.8)">
          <span>Free and open source</span>
          <span class="size-0.75 rounded-full bg-current" aria-hidden="true" />
          <span>Your clips never leave your PC</span>
        </Motion>

        <Motion
          class="w-full max-w-200"
          :initial="{ opacity: 0, y: 40, scale: 0.96 }"
          :animate="{ opacity: 1, y: 0, scale: 1 }"
          :transition="{ duration: 1, ease, delay: 1 }"
        >
          <AppPreview />
        </Motion>
      </section>

      <section id="features" class="flex flex-col items-center gap-12 w-full px-6">
        <h2 class="font-medium text-center text-5xl tracking-tight">Every copy becomes a card</h2>
        <p class="max-w-2xl mx-auto text-center text-neutral-600">
          Win+V forgets most of what you copy and shows the rest as a plain list. Micropaste keeps it all and shows each clip the way it deserves.
        </p>

        <FeatureGrid />
      </section>

      <section class="flex flex-col items-center gap-12 w-full px-6 mt-36">
        <Motion as="h2" class="font-medium text-center text-5xl tracking-tight" v-bind="reveal()">
          Reuse in one gesture
        </Motion>
        <Motion as="p" class="max-w-2xl mx-auto text-center text-neutral-600" v-bind="reveal(0.1)">
          Finding a clip is only half the job. Click it to copy it back, drag it into any app, or right-click for everything else. No retyping, no digging through folders.
        </Motion>

        <Motion class="w-full" v-bind="reveal(0.2, 0.3)">
          <ReuseGestures />
        </Motion>
      </section>

      <section id="collections" class="flex flex-col gap-12 w-full px-6 mt-36">
        <div class="grid gap-6 md:grid-cols-[3fr_2fr] md:items-end">
          <Motion as="h2" class="font-medium text-5xl tracking-tight" v-bind="reveal()">
            Your clips,
            <br>
            <span class="font-title text-neutral-400">filed your way</span>
          </Motion>
          <Motion as="p" class="text-neutral-600 md:pb-1.5" v-bind="reveal(0.1)">
            Group clips into your own collections, shown as tabs right next to your history.
          </Motion>
        </div>

        <Motion class="w-full" v-bind="reveal(0.2, 0.3)">
          <CollectionsShowcase />
        </Motion>

        <ol class="grid w-full gap-8 sm:grid-cols-2 lg:grid-cols-4">
          <Motion v-for="(fact, i) in collectionFacts" :key="fact.title" as="li" class="flex flex-col gap-2 border-t border-black/10 pt-5" v-bind="reveal(0.1 * i)">
            <span class="font-title text-sm text-neutral-400">0{{ i + 1 }}</span>
            <h3 class="text-lg font-medium tracking-tight">{{ fact.title }}</h3>
            <p class="text-neutral-500">{{ fact.description }}</p>
          </Motion>
        </ol>
      </section>

      <section id="windows" class="grid w-full gap-x-12 gap-y-6 px-6 mt-36 lg:grid-cols-[2fr_3fr]">
        <Motion as="h2" class="font-medium text-center text-5xl tracking-tight lg:col-span-2" v-bind="reveal()">
          Feels at home
          <br>
          <span class="font-title text-neutral-400">on Windows 11</span>
        </Motion>
        <Motion as="p" class="max-w-xl mx-auto mb-6 text-center text-neutral-600 lg:col-span-2" v-bind="reveal(0.1)">
          Built for Windows 11 and nothing else, so it looks and behaves like the rest of your desktop.
        </Motion>

        <ul class="mt-6 flex flex-col justify-between divide-y divide-black/6">
          <Motion v-for="(fact, i) in windowsFacts" :key="fact.title" as="li" class="flex gap-4 py-4 first:pt-0 last:pb-0" v-bind="reveal(0.2 + 0.1 * i)">
            <Icon :name="fact.icon" size="18" class="mt-0.5 shrink-0 text-neutral-400" />
            <div class="flex flex-col gap-1">
              <h3 class="font-medium tracking-tight">{{ fact.title }}</h3>
              <p class="text-sm text-neutral-500">{{ fact.description }}</p>
            </div>
          </Motion>
        </ul>

        <Motion class="lg:mt-6" v-bind="reveal(0.2, 0.3)">
          <WindowsShowcase />
        </Motion>
      </section>

      <DownloadCta />

      <section id="faq" class="grid w-full gap-x-12 gap-y-6 px-6 mt-36 lg:grid-cols-[2fr_3fr]">
        <div class="flex flex-col gap-6 lg:sticky lg:top-32 lg:self-start">
          <Motion as="h2" class="font-medium text-5xl tracking-tight" v-bind="reveal()">
            Questions,
            <br>
            <span class="font-title text-neutral-400">answered</span>
          </Motion>
          <Motion as="p" class="text-neutral-600" v-bind="reveal(0.1)">
            Something else on your mind?
            <NuxtLink :to="ISSUES_URL" target="_blank" class="font-medium text-neutral-800 underline underline-offset-4 decoration-neutral-400">Open an issue on GitHub</NuxtLink>.
          </Motion>
        </div>

        <FaqList />
      </section>
    </main>

    <Footer />
  </div>
</template>
