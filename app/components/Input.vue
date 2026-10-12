<script setup lang="ts">
defineOptions({ inheritAttrs: false })

withDefaults(defineProps<{
  icon?: string
  placeholder?: string
  type?: 'text' | 'search' | 'email' | 'url' | 'password'
}>(), {
  type: 'text'
})

const model = defineModel<string>({ default: '' })

const input = useTemplateRef<HTMLInputElement>('input')
defineExpose({ focus: () => input.value?.focus() })

const attrs = useAttrs()
const inputAttrs = computed(() => {
  const { class: _class, style: _style, ...rest } = attrs
  return rest
})
</script>

<template>
  <label
    class="flex h-6 cursor-text items-center gap-2 px-1"
    :class="$attrs.class"
    :style="$attrs.style"
  >
    <Icon v-if="icon" :name="icon" class="shrink-0 text-base opacity-60" />
    <input
      ref="input"
      v-model="model"
      v-bind="inputAttrs"
      class="h-full min-w-0 flex-1 bg-transparent p-0 font-ui text-sm outline-none placeholder:text-current placeholder:opacity-50"
      :type="type"
      :placeholder="placeholder"
    >
  </label>
</template>
