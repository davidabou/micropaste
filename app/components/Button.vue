<script setup lang="ts">
const props = withDefaults(defineProps<{
  icon?: string
  variant?: 'ghost' | 'subtle' | 'accent' | 'filled'
  size?: 'sm' | 'md'
  type?: 'button' | 'submit' | 'reset'
  pressed?: boolean
}>(), {
  variant: 'ghost',
  size: 'md',
  type: 'button'
})

const slots = useSlots()
const iconOnly = computed(() => !slots.default)

const variants = {
  ghost: 'hover:bg-black/[3.73%] active:bg-black/[2.41%] dark:hover:bg-white/[6.05%] dark:active:bg-white/[4.19%]',
  subtle: 'border border-black/[5.78%] bg-white/70 hover:bg-white/50 active:bg-white/30 dark:border-white/[9.3%] dark:bg-white/[6.05%] dark:hover:bg-white/[8.37%] dark:active:bg-white/[3.26%]',
  accent: 'bg-[#005fb8] text-white hover:bg-[#005fb8]/90 active:bg-[#005fb8]/80 dark:bg-[#60cdff] dark:text-black dark:hover:bg-[#60cdff]/90 dark:active:bg-[#60cdff]/80',
  filled: 'bg-black/[5.78%] hover:bg-black/[8%] active:bg-black/[4%] dark:bg-white/[8.37%] dark:hover:bg-white/[11%] dark:active:bg-white/[6%]'
}

const sizes = {
  sm: { base: 'h-7 gap-1.5', label: 'px-2.5', square: 'w-7' },
  md: { base: 'h-8 gap-2', label: 'px-3', square: 'w-8' }
}

const classes = computed(() => {
  const size = sizes[props.size]
  return [
    'inline-flex shrink-0 items-center justify-center rounded-md font-ui text-xs font-medium transition-colors duration-100',
    'focus-visible:outline-2 focus-visible:outline-current disabled:pointer-events-none disabled:opacity-40',
    variants[props.variant],
    size.base,
    iconOnly.value ? size.square : size.label,
    props.pressed && 'bg-black/[5.78%] dark:bg-white/[8.37%]'
  ]
})
</script>

<template>
  <button :type="type" :class="classes" :aria-pressed="pressed">
    <Icon v-if="icon" :name="icon" class="shrink-0 text-base" :class="pressed ? 'opacity-100' : 'opacity-60'" />
    <slot />
  </button>
</template>
