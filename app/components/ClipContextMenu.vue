<script setup lang="ts">
import type { Clip } from '@/composables/useClips'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'

const props = defineProps<{
  clip: Clip
  collection: number | null
}>()

const emit = defineEmits<{
  copy: []
  paste: []
  copyText: []
  save: []
  info: []
  favorite: []
  delete: []
  add: [collection: number]
  removeFrom: [collection: number]
}>()

const { collections } = useCollections()

const memberOf = ref<number[]>([])
const hasText = ref(false)
const pasteTarget = ref<{ name: string, icon: string | null } | null>(null)
const open = ref(false)
const subOpen = ref(false)
const opened = ref(false)
watch(open, async (isOpen) => {
  if (!isOpen) return
  opened.value = true
  invoke<{ name: string, icon: string | null } | null>('paste_target')
    .then(target => (pasteTarget.value = target))
    .catch(() => (pasteTarget.value = null))
  if (props.clip.kind !== 'text') {
    invoke<string | null>('get_image_text', { id: props.clip.id })
      .then(text => (hasText.value = !!text))
      .catch(() => (hasText.value = false))
  }
  memberOf.value = await invoke<number[]>('clip_collections', { clip: props.clip.id }).catch(() => [])
})

const saveLabel = computed(() => {
  if (props.clip.kind === 'image') return 'Save as…'
  if (props.clip.kind !== 'files') return null
  return props.clip.content.includes('\n') ? 'Save to folder…' : 'Save as…'
})

const link = computed(() => (props.clip.kind === 'text' ? copiedLink(props.clip.content) : null))
function openLink() {
  if (link.value) openUrl(link.value).catch(() => notifyClip(props.clip.id, 'Couldn’t open the link'))
}

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
            <ContextMenuItem v-if="pasteTarget" :class="menuItem" @select="emit('paste')">
              <span class="flex w-3.5 shrink-0 justify-center">
                <img v-if="pasteTarget.icon" :src="pasteTarget.icon" alt="" class="size-4 max-w-none" draggable="false">
                <Icon v-else name="gravity-ui:square-dashed" class="size-4 shrink-0 opacity-60" />
              </span>
              <span class="truncate">Paste to {{ pasteTarget.name }}</span>
            </ContextMenuItem>
            <ContextMenuItem v-if="hasText" :class="menuItem" @select="emit('copyText')">
              <Icon name="gravity-ui:square-dashed-text" :class="menuIcon" />
              Copy text
            </ContextMenuItem>
            <ContextMenuItem v-if="link" :class="menuItem" @select="openLink">
              <Icon name="gravity-ui:arrow-up-right-from-square" :class="menuIcon" />
              Open link
            </ContextMenuItem>
            <ContextMenuItem v-if="saveLabel" :class="menuItem" @select="emit('save')">
              <Icon name="gravity-ui:file-arrow-down" :class="menuIcon" />
              {{ saveLabel }}
            </ContextMenuItem>

            <ContextMenuSeparator :class="menuSeparator" />
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

            <ContextMenuSeparator :class="menuSeparator" />
            <ContextMenuItem v-if="collection === null" :class="menuItem" @select="emit('delete')">
              <Icon name="gravity-ui:trash-bin" :class="menuIcon" />
              Delete
            </ContextMenuItem>
            <ContextMenuItem v-else :class="menuItem" @select="emit('removeFrom', collection)">
              <Icon name="gravity-ui:xmark" :class="menuIcon" />
              Remove
            </ContextMenuItem>
          </Motion>
        </ContextMenuContent>
      </AnimatePresence>
    </ContextMenuPortal>
  </ContextMenuRoot>
</template>
