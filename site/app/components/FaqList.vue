<script setup lang="ts">
const questions = [
  {
    question: 'Is Micropaste free?',
    answer: 'Yes. It\'s free and open source under the MIT license, with no account, no trial and no paid tier.'
  },
  {
    question: 'Where are my clips stored?',
    answer: 'Only on your computer, in %APPDATA%\\com.abou.micropaste. There\'s no cloud, no sync and no telemetry.'
  },
  {
    question: 'Does it ever go online?',
    answer: 'Only to fetch the title and image of links you copy, so they can be shown as previews. Local and internal addresses are never fetched.'
  },
  {
    question: 'What about passwords?',
    answer: 'Password managers mark the secrets they copy so clipboard tools leave them alone. Micropaste respects that and never records them.'
  },
  {
    question: 'Does it replace Win+V?',
    answer: 'It sits next to it. Win+V keeps working, and Micropaste opens with Alt+Shift+V, or any shortcut you pick in the settings.'
  },
  {
    question: 'How long are clips kept?',
    answer: 'Clips you haven\'t used in 30 days are cleaned up, and you can choose anything from 1 day to forever. Favorites and clips in a collection are never removed.'
  },
  {
    question: 'Why does Windows warn me when I install it?',
    answer: 'The installer isn\'t signed yet, so SmartScreen doesn\'t recognize it. Click More info, then Run anyway.'
  },
  {
    question: 'Does it work on Windows 10?',
    answer: 'Micropaste is built and tested for Windows 11 only, and relies on its look and features.'
  }
]

useHead({
  script: [{
    type: 'application/ld+json',
    innerHTML: JSON.stringify({
      '@context': 'https://schema.org',
      '@type': 'FAQPage',
      'mainEntity': questions.map(item => ({
        '@type': 'Question',
        'name': item.question,
        'acceptedAnswer': { '@type': 'Answer', 'text': item.answer }
      }))
    })
  }]
})

const opened = ref<number | null>(null)

function toggle(i: number) {
  opened.value = opened.value === i ? null : i
}
</script>

<template>
  <ul class="flex flex-col divide-y divide-black/6 border-y border-black/6">
    <Motion v-for="(item, i) in questions" :key="item.question" as="li" v-bind="reveal(0.05 * i)">
      <h3>
        <button
          :id="`faq-question-${i}`"
          type="button"
          class="flex w-full items-center justify-between gap-6 py-5 text-left font-medium tracking-tight cursor-pointer"
          :aria-expanded="opened === i"
          :aria-controls="`faq-answer-${i}`"
          @click="toggle(i)"
        >
          {{ item.question }}

          <Motion
            class="flex shrink-0 text-neutral-400"
            :initial="false"
            :animate="{ rotate: opened === i ? 45 : 0 }"
            :transition="{ type: 'spring', bounce: 0.3, visualDuration: 0.3 }"
          >
            <Icon name="gravity-ui:plus" size="16" />
          </Motion>
        </button>
      </h3>

      <Motion
        :id="`faq-answer-${i}`"
        role="region"
        :aria-labelledby="`faq-question-${i}`"
        :inert="opened !== i"
        class="overflow-hidden"
        :initial="false"
        :animate="{ height: opened === i ? 'auto' : 0, opacity: opened === i ? 1 : 0 }"
        :transition="{ type: 'spring', bounce: 0, visualDuration: 0.35 }"
      >
        <p class="max-w-xl pb-5 text-neutral-500">{{ item.answer }}</p>
      </Motion>
    </Motion>
  </ul>
</template>
