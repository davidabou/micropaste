<script setup lang="ts">
import type { Tab } from '@/composables/useClips'
import type { Collection } from '@/composables/useCollections'
import { animate } from 'motion-v'

const emit = defineEmits<{
  prefetch: [tab: Exclude<Tab, 'history'>]
  dropClip: [collection: number]
}>()

const selected = defineModel<Tab>({ default: 'history' })

const { collections, historyCount, create, rename, reorder, remove } = useCollections()

const order = ref<Collection[]>([])
const draggingTab = ref(false)
watch(collections, (list) => {
  if (!draggingTab.value) order.value = [...list]
}, { immediate: true })
function saveOrder() {
  draggingTab.value = false
  const changed = order.value.some((collection, i) => collection.id !== collections.value[i]?.id)
  if (changed) reorder(order.value)
}

const dropTarget = ref<number | null>(null)
function onTileOver(event: DragEvent, id: number) {
  if (!tileDragging.value) return
  event.preventDefault()
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy'
  dropTarget.value = id
}
function onTileLeave(event: DragEvent, id: number) {
  const target = event.currentTarget as HTMLElement
  if (!target.contains(event.relatedTarget as Node | null) && dropTarget.value === id) dropTarget.value = null
}
function onTileDrop(event: DragEvent, id: number) {
  dropTarget.value = null
  if (!tileDragging.value) return
  event.preventDefault()
  emit('dropClip', id)
  animate(event.currentTarget as HTMLElement, { scale: [1, 1.15, 1] }, { duration: 0.4, ease: 'easeOut' })
}

const tab = computed({
  get: () => String(selected.value),
  set: (value: string) => (selected.value = value === 'history' || value === 'favorites' ? value : Number(value))
})

const menuOpenFor = ref<number | null>(null)

const editing = ref<'new' | number | null>(null)
const draft = ref('')
let nameInput: HTMLInputElement | null = null
// Gets the <input> itself, from a plain input or a Motion one (a component)
function setNameInput(el: unknown) {
  nameInput = ((el as { $el?: HTMLInputElement } | null)?.$el ?? el) as HTMLInputElement | null
}

const renameFromWidth = ref(0)

async function startEditing(target: 'new' | number, name = '') {
  if (target !== 'new') {
    renameFromWidth.value = document.querySelector<HTMLElement>(`[data-collection-tab="${target}"]`)?.offsetWidth ?? 0
  }
  draft.value = name
  await nextTick()
  await measureName()
  editing.value = target
  await nextTick()
  nameInput?.focus()
  nameInput?.select()
}

async function commit() {
  const target = editing.value
  const name = draft.value.trim()
  editing.value = null
  if (target === null || !name) return
  if (target === 'new') {
    selected.value = (await create(name)).id
  } else {
    await rename(target, name)
  }
}
const cancel = () => (editing.value = null)

const confirmingDelete = ref<number | null>(null)
const deleteBubbleFor = ref<number | null>(null)
const deleteFromWidth = ref(0)
const deleteToWidth = ref(0)
function setDeleteContent(el: unknown) {
  const node = ((el as { $el?: HTMLElement } | null)?.$el ?? el) as HTMLElement | null
  if (node) deleteToWidth.value = node.offsetWidth
}
async function removeCollection(id: number) {
  const collection = collections.value.find(c => c.id === id)
  if (collection?.count) {
    deleteFromWidth.value = document.querySelector<HTMLElement>(`[data-collection-tab="${id}"]`)?.offsetWidth ?? 0
    deleteToWidth.value = 0
    confirmingDelete.value = id
    deleteBubbleFor.value = id
    return
  }
  await deleteCollection(id)
}
async function confirmDelete(id: number) {
  await deleteCollection(id)
  confirmingDelete.value = null
  deleteBubbleFor.value = null
}
const confirming = () => (confirmingDelete.value === null ? null : window)
onKeyStroke('Escape', (event) => {
  event.preventDefault()
  confirmingDelete.value = null
}, { target: confirming })
useEventListener(confirming, 'pointerdown', (event: PointerEvent) => {
  if (!(event.target as Element | null)?.closest('[data-collection-delete-confirm]')) confirmingDelete.value = null
}, { capture: true })
function onDeleteBubbleAnimated() {
  if (confirmingDelete.value === null) deleteBubbleFor.value = null
}
async function deleteCollection(id: number) {
  if (selected.value === id) selected.value = 'history'
  await remove(id)
}
const textButton = 'cursor-pointer rounded-sm text-xs font-medium transition-opacity duration-100 hover:opacity-60 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-current'
const tabLabelHidden = { opacity: 0, scale: 0.9, transition: { duration: 0.12 } }
const tabLabelShown = { opacity: 1, scale: 1, transition: { duration: 0.15, delay: 0.15 } }

// No overflow clipping on the tab itself (the name is truncated inside): the sliding pill would be cut
const triggerBase = 'relative isolate flex h-8 items-center shrink-0 cursor-pointer rounded-md text-sm opacity-60 transition-opacity duration-100 hover:opacity-100 focus-visible:outline-2 focus-visible:outline-current data-[state=active]:opacity-100'
const trigger = `${triggerBase} max-w-48 px-3`
const iconTrigger = `${triggerBase} w-8 justify-center`

const creating = computed(() => editing.value === 'new')
const morph = { type: 'spring', bounce: 0.2, duration: 0.4 } as const
const NAME_PLACEHOLDER = 'Name...'
const nameMirror = useTemplateRef<HTMLElement>('nameMirror')
const nameWidth = ref(0)
async function measureName() {
  if (!nameMirror.value) return
  nameWidth.value = Math.min(192, Math.max(48, Math.ceil(nameMirror.value.offsetWidth) + 2))
  // The field may have scrolled to show the caret just before it widened: back to the start
  await nextTick()
  if (nameInput && nameInput.scrollWidth <= nameInput.clientWidth) nameInput.scrollLeft = 0
}
onMounted(measureName)
watch(draft, measureName, { flush: 'post' })
const morphWidth = computed(() => ({ width: creating.value ? nameWidth.value : 32 }))
const pillMorph = { type: 'spring', bounce: 0.15, visualDuration: 0.25 } as const
const plusMotion = {
  initial: { opacity: 0, scale: 0.6 },
  animate: { opacity: 1, scale: 1, transition: { type: 'spring' as const, bounce: 0.45, visualDuration: 0.3 } },
  exit: { opacity: 0, scale: 0.6, transition: { duration: 0.08, ease: 'easeIn' as const } }
}
const fieldMotion = {
  initial: { opacity: 0, x: -6 },
  animate: { opacity: 1, x: 0, transition: { type: 'spring' as const, bounce: 0, visualDuration: 0.2, delay: 0.04 } },
  exit: { opacity: 0, transition: { duration: 0.08 } }
}
const newNameText = 'h-8 shrink-0 bg-transparent px-3 text-sm outline-none placeholder:text-current placeholder:opacity-50'

const tabPop = {
  initial: { opacity: 0, scale: 0.6 },
  animate: { opacity: 1, scale: 1, transition: { type: 'spring' as const, bounce: 0.5, duration: 0.45 } },
  exit: { opacity: [1, 1, 0], scale: [1, 1.12, 0.3], transition: { duration: 0.3, times: [0, 0.3, 1], ease: 'easeIn' as const } }
}
const tabSlide = { type: 'spring', bounce: 0.2, duration: 0.4 } as const
</script>

<template>
  <TabsRoot v-model="tab">
    <ScrollArea orientation="horizontal">
      <TabsList class="relative flex items-center gap-1 px-2 pb-1">
        <LayoutGroup>
          <Tooltip content="Favorites">
            <TabsTrigger value="favorites" aria-label="Favorites" :class="iconTrigger" @pointerenter="emit('prefetch', 'favorites')">
              <Icon name="gravity-ui:star-fill" class="shrink-0 text-base" />
              <TabPill v-if="tab === 'favorites'" />
            </TabsTrigger>
          </Tooltip>
          <TabsTrigger value="history" :class="trigger">
            <span class="flex min-w-0 items-baseline gap-1.5">
              <span class="min-w-0 truncate">History</span>
              <TabCount :value="historyCount" />
            </span>
            <TabPill v-if="tab === 'history'" />
          </TabsTrigger>

          <ReorderGroup
            as="div"
            axis="x"
            class="flex items-center gap-1"
            :values="order"
            @update:values="order = $event"
          >
            <AnimatePresence :initial="false">
              <ReorderItem
                v-for="collection in order"
                :key="collection.id"
                :value="collection"
                as="div"
                layout="position"
                class="shrink-0 rounded-md transition-colors duration-100"
                :class="dropTarget === collection.id && 'bg-black/8 dark:bg-white/11'"
                :drag="editing === collection.id || deleteBubbleFor === collection.id ? false : 'x'"
                :initial="tabPop.initial"
                :animate="tabPop.animate"
                :exit="tabPop.exit"
                :transition="tabSlide"
                @drag-start="draggingTab = true"
                @drag-end="saveOrder"
                @dragover="onTileOver($event, collection.id)"
                @dragleave="onTileLeave($event, collection.id)"
                @drop="onTileDrop($event, collection.id)"
              >
                <Motion
                  v-if="deleteBubbleFor === collection.id"
                  data-collection-delete-confirm
                  class="relative flex h-8 items-center overflow-hidden bg-black/[5.78%] dark:bg-white/[8.37%]"
                  :style="{ borderRadius: '6px' }"
                  :initial="{ width: deleteFromWidth }"
                  :animate="{ width: confirmingDelete === collection.id ? deleteToWidth || deleteFromWidth : deleteFromWidth }"
                  :transition="morph"
                  @animation-complete="onDeleteBubbleAnimated"
                >
                  <Motion
                    class="pointer-events-none absolute inset-y-0 left-0 flex items-center px-3 text-sm"
                    :initial="{ opacity: 1, scale: 1 }"
                    :animate="confirmingDelete === collection.id ? tabLabelHidden : tabLabelShown"
                  >
                    <span class="flex items-baseline gap-1.5 whitespace-nowrap" :class="tab !== String(collection.id) && 'opacity-60'">
                      <span class="max-w-36 truncate">{{ collection.name }}</span>
                      <TabCount :value="collection.count" />
                    </span>
                  </Motion>
                  <Motion
                    v-if="confirmingDelete === collection.id"
                    :ref="setDeleteContent"
                    class="flex w-max shrink-0 items-center gap-3 px-3 whitespace-nowrap"
                    :initial="{ opacity: 0 }"
                    :animate="{ opacity: 1, transition: { delay: 0.1, duration: 0.15 } }"
                  >
                    <span class="text-sm">Delete “{{ collection.name }}”?</span>
                    <button type="button" :class="textButton" @click="confirmingDelete = null">
                      Cancel
                    </button>
                    <button type="button" :class="textButton" @click="confirmDelete(collection.id)">
                      Delete
                    </button>
                  </Motion>
                </Motion>
                <Motion
                  v-else-if="editing === collection.id"
                  :initial="{ width: renameFromWidth || nameWidth }"
                  :animate="{ width: nameWidth }"
                  :transition="morph"
                  :style="{ borderRadius: '6px' }"
                  class="flex h-8 overflow-hidden bg-black/[5.78%] dark:bg-white/[8.37%]"
                >
                  <input
                    :ref="setNameInput"
                    v-model="draft"
                    :class="newNameText"
                    :style="{ width: `${nameWidth}px` }"
                    spellcheck="false"
                    @keydown.enter="commit"
                    @keydown.esc.prevent="cancel"
                    @blur="commit"
                  >
                </Motion>
                <ContextMenuRoot
                  v-else
                  :open="menuOpenFor === collection.id"
                  @update:open="menuOpenFor = $event ? collection.id : null"
                >
                  <ContextMenuTrigger as-child>
                    <TabsTrigger :value="String(collection.id)" :data-collection-tab="collection.id" :class="trigger" @pointerenter="emit('prefetch', collection.id)">
                      <span class="flex min-w-0 items-baseline gap-1.5">
                        <span class="min-w-0 truncate">{{ collection.name }}</span>
                        <TabCount :value="collection.count" />
                      </span>
                      <TabPill v-if="tab === String(collection.id)" />
                    </TabsTrigger>
                  </ContextMenuTrigger>
                  <ContextMenuPortal force-mount>
                    <AnimatePresence>
                      <ContextMenuContent
                        v-if="menuOpenFor === collection.id"
                        force-mount
                        as-child
                        @close-auto-focus.prevent
                      >
                        <Motion :class="menuContent" v-bind="menuMotion">
                          <ContextMenuItem :class="menuItem" @select="startEditing(collection.id, collection.name)">
                            <Icon name="gravity-ui:pencil" :class="menuIcon" />
                            Rename
                          </ContextMenuItem>
                          <ContextMenuItem :class="menuItem" @select="removeCollection(collection.id)">
                            <Icon name="gravity-ui:trash-bin" :class="menuIcon" />
                            Delete collection
                          </ContextMenuItem>
                        </Motion>
                      </ContextMenuContent>
                    </AnimatePresence>
                  </ContextMenuPortal>
                </ContextMenuRoot>
              </ReorderItem>
            </AnimatePresence>
          </ReorderGroup>

          <Tooltip content="New collection" :disabled="creating">
            <Motion
              layout="position"
              :initial="false"
              :animate="morphWidth"
              :transition="pillMorph"
              :style="{ borderRadius: '6px' }"
              class="relative flex h-8 shrink-0 items-center overflow-hidden transition-colors duration-100"
              :class="creating ? 'bg-black/[5.78%] dark:bg-white/[8.37%]' : 'hover:bg-black/[3.73%] dark:hover:bg-white/[6.05%]'"
            >
              <span
                ref="nameMirror"
                aria-hidden="true"
                class="pointer-events-none invisible absolute left-0 w-max px-3 text-sm whitespace-pre"
              >{{ draft || NAME_PLACEHOLDER }}</span>
              <AnimatePresence mode="popLayout" :initial="false">
                <Motion
                  v-if="!creating"
                  key="button"
                  as="button"
                  type="button"
                  class="flex size-8 shrink-0 cursor-pointer items-center justify-center focus-visible:outline-2 focus-visible:outline-current"
                  v-bind="plusMotion"
                  @click="startEditing('new')"
                >
                  <Icon name="gravity-ui:plus" class="text-base opacity-70" />
                </Motion>
                <Motion
                  v-else
                  key="field"
                  :ref="setNameInput"
                  as="input"
                  :value="draft"
                  :class="newNameText"
                  :style="{ width: `${nameWidth}px` }"
                  :placeholder="NAME_PLACEHOLDER"
                  spellcheck="false"
                  v-bind="fieldMotion"
                  @input="draft = ($event.target as HTMLInputElement).value"
                  @keydown.enter="commit"
                  @keydown.esc.prevent="cancel"
                  @blur="commit"
                />
              </AnimatePresence>
            </Motion>
          </Tooltip>
        </LayoutGroup>
      </TabsList>
    </ScrollArea>
  </TabsRoot>
</template>
