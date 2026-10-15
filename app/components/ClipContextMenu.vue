<script setup lang="ts">
import type { Clip } from '@/composables/useClips'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  clip: Clip
  collection: number | null
}>()

const emit = defineEmits<{
  copy: []
  info: []
  favorite: []
  delete: []
  add: [collection: number]
  removeFrom: [collection: number]
}>()

const { collections } = useCollections()

const memberOf = ref<number[]>([])
const open = ref(false)
const subOpen = ref(false)
const opened = ref(false)
watch(open, async (isOpen) => {
  if (isOpen) opened.value = true
  if (isOpen) memberOf.value = await invoke<number[]>('clip_collections', { clip: props.clip.id }).catch(() => [])
})

function toggle(collection: number, checked: boolean | 'indeterminate') {
  if (checked === true) emit('add', collection)
  else emit('removeFrom', collection)
}
</script>

<template>
  <ContextMenuRoot v-model:open="open">
    <ContextMenuTrigger as-child>
      <slot />
    </ContextMenuTrigger>
    <ContextMenuPortal v-if="opened" force-mount>
      <AnimatePresence>
        <ContextMenuContent v-if="open" force-mount as-child>
          <Motion :class="menuContent" v-bind="menuMotion">
            <ContextMenuItem :class="menuItem" @select="emit('copy')">
              <Icon name="gravity-ui:copy" :class="menuIcon" />
              Copy
            </ContextMenuItem>
            <ContextMenuItem :class="menuItem" @select="emit('info')">
              <Icon name="gravity-ui:circle-info" :class="menuIcon" />
              Details
            </ContextMenuItem>
            <ContextMenuItem :class="menuItem" @select="emit('favorite')">
              <Icon :name="clip.favorite ? 'gravity-ui:star-fill' : 'gravity-ui:star'" :class="menuIcon" />
              Favorite
            </ContextMenuItem>

            <ContextMenuSub v-model:open="subOpen">
              <ContextMenuSubTrigger :class="menuItem">
                <Icon name="gravity-ui:folder" :class="menuIcon" />
                Collections
                <Icon name="gravity-ui:chevron-right" class="ml-auto text-xs opacity-60" />
              </ContextMenuSubTrigger>
              <ContextMenuPortal force-mount>
                <AnimatePresence>
                  <ContextMenuSubContent v-if="subOpen" force-mount as-child :side-offset="4">
                    <Motion :class="menuContent" v-bind="menuMotion">
                      <ContextMenuCheckboxItem
                        v-for="item in collections"
                        :key="item.id"
                        :class="menuItem"
                        :model-value="memberOf.includes(item.id)"
                        @update:model-value="toggle(item.id, $event)"
                      >
                        <span class="flex w-3 items-center justify-center">
                          <ContextMenuItemIndicator class="flex">
                            <Icon name="gravity-ui:check" class="text-sm" />
                          </ContextMenuItemIndicator>
                        </span>
                        <span class="truncate">{{ item.name }}</span>
                      </ContextMenuCheckboxItem>
                      <ContextMenuItem v-if="!collections.length" :class="menuItem" disabled>
                        No collection yet
                      </ContextMenuItem>
                    </Motion>
                  </ContextMenuSubContent>
                </AnimatePresence>
              </ContextMenuPortal>
            </ContextMenuSub>

            <ContextMenuItem v-if="collection === null" :class="menuItem" @select="emit('delete')">
              <Icon name="gravity-ui:trash-bin" :class="menuIcon" />
              Delete
            </ContextMenuItem>
          </Motion>
        </ContextMenuContent>
      </AnimatePresence>
    </ContextMenuPortal>
  </ContextMenuRoot>
</template>
