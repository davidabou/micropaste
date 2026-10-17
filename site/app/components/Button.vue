<script setup lang="ts">
const { variant = 'primary', size = 'md', iconSize } = defineProps<{
  to: string
  icon?: string
  iconSize?: number
  variant?: 'primary' | 'ghost'
  size?: 'sm' | 'md'
}>()

const sizes = {
  sm: 'h-8 px-3 gap-1.5 text-sm',
  md: 'h-10 px-4 gap-2.5'
}

const variants = {
  primary: 'text-white bg-accent border border-accent hover:bg-accent-hover inset-shadow-[0_1.5px_--theme(--color-white/18%)] shadow-black/24 shadow-xs active:inset-shadow-[0_1.5px_--theme(--color-black/8%)] active:shadow-none transition-shadow',
  ghost: 'transition-colors hover:bg-black/[3.73%]'
}
</script>

<template>
  <NuxtLink :to="to" target="_blank" class="flex items-center w-fit rounded-lg font-medium" :class="[sizes[size], variants[variant]]">
    <Icon v-if="icon" :name="icon" :size="String(iconSize ?? (size === 'md' ? 20 : 18))" />

    <span><slot /></span>
  </NuxtLink>
</template>
