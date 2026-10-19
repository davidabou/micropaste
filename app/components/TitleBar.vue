<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'

const close = () => getCurrentWindow().close()
</script>

<template>
  <header class="flex h-12 items-center gap-1 px-2 select-none" data-tauri-drag-region>
    <div class="flex min-w-0 flex-1 items-center gap-2" data-tauri-drag-region>
      <slot />
    </div>

    <Tooltip content="Close">
      <Button icon="gravity-ui:xmark" size="sm" aria-label="Close" @click="close" />
    </Tooltip>
  </header>
</template>
