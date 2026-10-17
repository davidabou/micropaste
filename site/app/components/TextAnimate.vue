<script setup lang="ts">
const { text, delay = 0, stagger = 0.05 } = defineProps<{
  text: string
  delay?: number
  stagger?: number
}>()

const words = computed(() => {
  let index = 0
  return text.split(' ').map(word => [...word].map(char => ({ char, index: index++ })))
})
</script>

<template>
  <span>
    <span class="sr-only">{{ text }}</span>

    <span aria-hidden="true">
      <template v-for="(word, w) in words" :key="w">
        <span class="inline-block whitespace-nowrap">
          <Motion
            v-for="{ char, index } in word"
            :key="index"
            as="span"
            class="inline-block"
            :initial="{ opacity: 0, y: 20, filter: 'blur(10px)' }"
            :animate="{ opacity: 1, y: 0, filter: 'blur(0px)' }"
            :transition="{ duration: 0.4, ease: 'easeOut', delay: delay + index * stagger }"
          >{{ char }}</Motion>
        </span>
        {{ ' ' }}
      </template>
    </span>
  </span>
</template>
