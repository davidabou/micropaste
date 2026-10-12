<script setup lang="ts">
import { animate } from 'motion-v'
import type { Collection } from '~/composables/useCollections'

const selected = defineModel<number | null>({ default: null })

const emit = defineEmits<{
  prefetch: [collection: number]
  dropClip: [collection: number]
}>()

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

const formatCount = (count: number) => count.toLocaleString('en-US')

const tab = computed({
  get: () => (selected.value === null ? 'history' : String(selected.value)),
  set: (value: string) => (selected.value = value === 'history' ? null : Number(value))
})

const menuOpenFor = ref<number | null>(null)

const editing = ref<'new' | number | null>(null)
const draft = ref('')
let nameInput: HTMLInputElement | null = null
// Gets the <input> itself, from a plain input or a Motion one (a component)
const setNameInput = (el: unknown) => {
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

const deleting = ref<Collection | null>(null)
// Kept apart: the dialog clears `deleting` as it closes, before the Delete click is handled
let deletingId: number | null = null
async function removeCollection(id: number) {
  const collection = collections.value.find(c => c.id === id)
  if (collection?.count) {
    deleting.value = collection
    deletingId = id
    return
  }
  await deleteCollection(id)
}
function confirmDeleteCollection() {
  if (deletingId !== null) deleteCollection(deletingId)
  deletingId = null
}
async function deleteCollection(id: number) {
  deleting.value = null
  if (selected.value === id) selected.value = null
  await remove(id)
}
const clipCount = (count: number) => `${formatCount(count)} ${count === 1 ? 'clip' : 'clips'}`

const dialogMotion = {
  initial: { opacity: 0, scale: 0.9 },
  animate: { opacity: 1, scale: 1, transition: { type: 'spring' as const, bounce: 0.3, visualDuration: 0.22 } },
  exit: { opacity: 0, scale: 0.95, transition: { type: 'spring' as const, bounce: 0, visualDuration: 0.12 } }
}

// No overflow clipping on the tab itself (the name is truncated inside): the sliding pill would be cut
const trigger = 'relative isolate flex h-8 items-center max-w-48 shrink-0 cursor-pointer rounded-md px-3 text-sm font-medium opacity-60 transition-opacity duration-100 hover:opacity-100 focus-visible:outline-2 focus-visible:outline-current data-[state=active]:opacity-100'

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
const newNameText = 'h-8 shrink-0 bg-transparent px-3 font-ui text-sm font-medium outline-none placeholder:text-current placeholder:opacity-50'

const tabPop = {
  initial: { opacity: 0, scale: 0.6 },
  animate: { opacity: 1, scale: 1, transition: { type: 'spring' as const, bounce: 0.5, duration: 0.45 } },
  exit: { opacity: [1, 1, 0], scale: [1, 1.12, 0.3], transition: { duration: 0.3, times: [0, 0.3, 1], ease: 'easeIn' as const } }
}
const tabSlide = { type: 'spring', bounce: 0.2, duration: 0.4 } as const
</script>

<template>
  <TabsRoot v-model="tab">
    <TabsList class="relative flex items-center gap-1 overflow-x-auto px-2 pb-1">
      <LayoutGroup>
        <TabsTrigger value="history" :class="trigger">
          <span class="flex min-w-0 items-baseline gap-1.5">
            <span class="min-w-0 truncate">History</span>
            <span class="shrink-0 text-xs font-normal tabular-nums opacity-60">{{ formatCount(historyCount) }}</span>
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
              :drag="editing === collection.id ? false : 'x'"
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
                v-if="editing === collection.id"
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
                  @keydown.esc="cancel"
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
                      <span class="shrink-0 text-xs font-normal tabular-nums opacity-60">{{ formatCount(collection.count) }}</span>
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
            :transition="morph"
            :style="{ borderRadius: '6px' }"
            class="relative flex h-8 shrink-0 items-center overflow-hidden transition-colors duration-100"
            :class="creating ? 'bg-black/[5.78%] dark:bg-white/[8.37%]' : 'hover:bg-black/[3.73%] dark:hover:bg-white/[6.05%]'"
          >
            <span
              ref="nameMirror"
              aria-hidden="true"
              class="pointer-events-none invisible absolute left-0 w-max px-3 font-ui text-sm font-medium whitespace-pre"
            >{{ draft || NAME_PLACEHOLDER }}</span>
            <AnimatePresence mode="popLayout" :initial="false">
              <Motion
                v-if="!creating"
                key="button"
                as="button"
                type="button"
                class="flex size-8 shrink-0 cursor-pointer items-center justify-center focus-visible:outline-2 focus-visible:outline-current"
                :initial="{ opacity: 0, scale: 0.5, rotate: -90 }"
                :animate="{ opacity: 1, scale: 1, rotate: 0 }"
                :exit="{ opacity: 0, scale: 0.5, rotate: 90 }"
                :transition="morph"
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
                :initial="{ opacity: 0 }"
                :animate="{ opacity: 1 }"
                :exit="{ opacity: 0 }"
                :transition="{ duration: 0.15 }"
                @input="draft = ($event.target as HTMLInputElement).value"
                @keydown.enter="commit"
                @keydown.esc="cancel"
                @blur="commit"
              />
            </AnimatePresence>
          </Motion>
        </Tooltip>
      </LayoutGroup>
    </TabsList>

    <AlertDialogRoot :open="deleting !== null" @update:open="open => !open && (deleting = null)">
      <AlertDialogPortal force-mount>
        <AnimatePresence>
          <AlertDialogOverlay v-if="deleting" key="overlay" force-mount as-child>
            <Motion
              class="fixed inset-0 z-50 bg-black/30"
              :initial="{ opacity: 0 }"
              :animate="{ opacity: 1 }"
              :exit="{ opacity: 0 }"
              :transition="{ duration: 0.15 }"
            />
          </AlertDialogOverlay>
          <AlertDialogContent v-if="deleting" key="content" force-mount as-child>
            <Motion
              v-bind="dialogMotion"
              class="fixed top-1/2 left-1/2 z-50 w-88 -translate-x-1/2 -translate-y-1/2 rounded-lg border border-black/[5.78%] bg-[#f9f9f9] p-5 font-ui shadow-xl dark:border-black/20 dark:bg-[#2c2c2c]"
            >
              <AlertDialogTitle class="truncate text-base font-semibold">
                Delete “{{ deleting.name }}”?
              </AlertDialogTitle>
              <AlertDialogDescription class="mt-2 text-sm opacity-80">
                Its {{ clipCount(deleting.count) }} will stay in your history.
              </AlertDialogDescription>
              <div class="mt-5 flex justify-end gap-2">
                <AlertDialogCancel as-child>
                  <Button>Cancel</Button>
                </AlertDialogCancel>
                <AlertDialogAction as-child>
                  <Button variant="filled" @click="confirmDeleteCollection">
                    Delete
                  </Button>
                </AlertDialogAction>
              </div>
            </Motion>
          </AlertDialogContent>
        </AnimatePresence>
      </AlertDialogPortal>
    </AlertDialogRoot>
  </TabsRoot>
</template>
