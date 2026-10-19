<script setup lang="ts">
const props = defineProps<{
  /** `null` while the file is being allowed: the controls wait for it */
  src: string | null
  video?: boolean
  poster?: string | null
}>()

const emit = defineEmits<{
  error: []
}>()

const root = useTemplateRef<HTMLElement>('root')
const media = useTemplateRef<HTMLMediaElement>('media')
const { playing, currentTime, duration, volume, muted, rate, onSourceError } = useMediaControls(media, { src: () => props.src ?? [] })
onSourceError(() => emit('error'))
const { isFullscreen, toggle: toggleFullscreen } = useFullscreen(root)

const controls = useTemplateRef<HTMLElement>('controls')
const controlsHovered = useElementHover(controls)
const awake = ref(true)
const { start: sleepLater } = useTimeoutFn(() => (awake.value = false), 2000, { immediate: false })
function wake() {
  awake.value = true
  sleepLater()
}
watch(playing, isPlaying => isPlaying && wake())
const controlsShown = computed(() => !playing.value || awake.value || controlsHovered.value)

const savedVolume = useLocalStorage('media-volume', 1)
volume.value = savedVolume.value
watch(volume, value => (savedVolume.value = value))

const RATES = [1, 1.25, 1.5, 2, 0.75]
const nextRate = () => (rate.value = RATES[(RATES.indexOf(rate.value) + 1) % RATES.length]!)

const position = computed({
  get: () => [currentTime.value],
  set: value => (currentTime.value = value?.[0] ?? 0)
})
const volumeLevel = computed({
  get: () => [muted.value ? 0 : volume.value],
  set: (value) => {
    volume.value = value?.[0] ?? 0
    muted.value = volume.value === 0
  }
})
const volumeIcon = computed(() => {
  if (muted.value || volume.value === 0) return 'gravity-ui:volume-xmark-fill'
  return volume.value < 0.5 ? 'gravity-ui:volume-low-fill' : 'gravity-ui:volume-fill'
})

function formatTime(seconds: number) {
  const total = Math.floor(Number.isFinite(seconds) ? seconds : 0)
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`
}

const sliderRoot = 'group/slider relative flex cursor-pointer touch-none items-center select-none'
const sliderTrack = 'relative h-1 grow overflow-hidden rounded-full bg-black/[8%] dark:bg-white/[12%]'
const sliderRange = 'absolute h-full bg-black/55 transition-colors duration-100 group-hover/slider:bg-black/80 group-has-focus-visible/slider:bg-black/80 dark:bg-white/70 dark:group-hover/slider:bg-white/95 dark:group-has-focus-visible/slider:bg-white/95'
const sliderThumb = 'block size-3 cursor-pointer rounded-full border border-black/[10%] bg-white opacity-0 shadow-sm transition-opacity duration-100 group-hover/slider:opacity-100 focus-visible:opacity-100 focus-visible:outline-2 focus-visible:outline-current dark:border-white/10 dark:bg-[#454545]'
// Over the video: white on a dark gradient, whatever the theme
const videoTrack = 'relative h-1 grow overflow-hidden rounded-full bg-white/30'
const videoRange = 'absolute h-full bg-white/80 transition-colors duration-100 group-hover/slider:bg-white group-has-focus-visible/slider:bg-white'
const videoThumb = 'block size-3 cursor-pointer rounded-full bg-white opacity-0 shadow-sm transition-opacity duration-100 group-hover/slider:opacity-100 focus-visible:opacity-100 focus-visible:outline-2 focus-visible:outline-white'
const videoButton = 'flex h-7 min-w-7 shrink-0 cursor-pointer items-center justify-center rounded-md opacity-90 transition-[opacity,background-color] duration-100 hover:bg-white/15 hover:opacity-100 focus-visible:outline-2 focus-visible:outline-white'
</script>

<template>
  <div
    ref="root"
    :class="video ? ['relative', isFullscreen && 'bg-black'] : 'flex items-center gap-4'"
    @keydown.left.stop
    @keydown.right.stop
    @pointermove="video && wake()"
    @pointerleave="awake = false"
  >
    <template v-if="video">
      <video
        ref="media"
        :poster="poster ?? undefined"
        class="size-full object-contain"
        :class="controlsShown ? 'cursor-pointer' : 'cursor-none'"
        @click="playing = !playing"
        @dblclick="toggleFullscreen"
      />
    </template>
    <audio v-else ref="media" />

    <div
      v-if="video"
      ref="controls"
      class="absolute inset-x-0 bottom-0 flex flex-col bg-linear-to-t from-black/70 via-black/35 to-transparent px-2 pt-8 pb-1 text-white transition-opacity duration-150"
      :class="!controlsShown && 'opacity-0 has-focus-visible:opacity-100'"
    >
      <div class="flex items-center gap-0.5">
        <button type="button" :class="videoButton" :aria-label="playing ? 'Pause' : 'Play'" @click="playing = !playing">
          <Icon :name="playing ? 'gravity-ui:pause-fill' : 'gravity-ui:play-fill'" class="text-base" />
        </button>
        <span class="ml-1 flex-1 text-xs tabular-nums">{{ formatTime(currentTime) }} / {{ formatTime(duration) }}</span>
        <div class="group/volume flex items-center">
          <SliderRoot
            v-model="volumeLevel"
            :max="1"
            :step="0.01"
            class="h-7 w-0 opacity-0 transition-[width,opacity,margin] duration-150 group-focus-within/volume:mr-1 group-focus-within/volume:w-14 group-focus-within/volume:opacity-100 group-hover/volume:mr-1 group-hover/volume:w-14 group-hover/volume:opacity-100"
            :class="[sliderRoot]"
          >
            <SliderTrack :class="videoTrack">
              <SliderRange :class="videoRange" />
            </SliderTrack>
            <SliderThumb :class="videoThumb" aria-label="Volume" />
          </SliderRoot>
          <button type="button" :class="videoButton" :aria-label="muted ? 'Unmute' : 'Mute'" @click="muted = !muted">
            <Icon :name="volumeIcon" class="text-base" />
          </button>
        </div>
        <button type="button" class="px-1.5 text-xs tabular-nums" :class="videoButton" aria-label="Playback speed" @click="nextRate">
          {{ rate }}×
        </button>
        <button
          type="button"
          :class="videoButton"
          :aria-label="isFullscreen ? 'Exit full screen' : 'Full screen'"
          @click="toggleFullscreen"
        >
          <Icon :name="isFullscreen ? 'gravity-ui:chevrons-collapse-up-right' : 'gravity-ui:chevrons-expand-up-right'" class="text-base" />
        </button>
      </div>
      <SliderRoot
        v-model="position"
        :max="duration || 1"
        :step="0.1"
        :disabled="!duration"
        class="mx-1 h-3" :class="[sliderRoot]"
      >
        <SliderTrack :class="videoTrack">
          <SliderRange :class="videoRange" />
        </SliderTrack>
        <SliderThumb :class="videoThumb" aria-label="Position" />
      </SliderRoot>
    </div>

    <template v-else>
      <div class="flex size-28 shrink-0 items-center justify-center overflow-hidden rounded-md bg-black/[5.78%] shadow-md ring-1 ring-black/10 dark:bg-white/[8.37%] dark:ring-white/10">
        <img v-if="poster" :src="poster" alt="" class="animate-fade-in size-full object-cover">
        <Icon v-else name="gravity-ui:music-note" class="text-[40px] opacity-40" />
      </div>
      <div class="flex min-w-0 flex-1 flex-col">
        <div class="-mx-1 flex items-center gap-0.5">
          <Button
            variant="filled"
            size="sm"
            :icon="playing ? 'gravity-ui:pause-fill' : 'gravity-ui:play-fill'"
            :aria-label="playing ? 'Pause' : 'Play'"
            :disabled="!src"
            @click="playing = !playing"
          />
          <div class="group/volume flex items-center">
            <Button size="sm" :icon="volumeIcon" :aria-label="muted ? 'Unmute' : 'Mute'" @click="muted = !muted" />
            <SliderRoot
              v-model="volumeLevel"
              :max="1"
              :step="0.01"
              class="h-7 w-0 opacity-0 transition-[width,opacity,margin] duration-150 group-focus-within/volume:mx-1 group-focus-within/volume:w-14 group-focus-within/volume:opacity-100 group-hover/volume:mx-1 group-hover/volume:w-14 group-hover/volume:opacity-100"
              :class="[sliderRoot]"
            >
              <SliderTrack :class="sliderTrack">
                <SliderRange :class="sliderRange" />
              </SliderTrack>
              <SliderThumb :class="sliderThumb" aria-label="Volume" />
            </SliderRoot>
          </div>
          <Tooltip content="Playback speed">
            <Button size="sm" class="ml-auto min-w-9 tabular-nums" aria-label="Playback speed" @click="nextRate">
              {{ rate }}×
            </Button>
          </Tooltip>
        </div>
        <SliderRoot
          v-model="position"
          :max="duration || 1"
          :step="0.1"
          :disabled="!duration"
          class="mt-1 h-5" :class="[sliderRoot]"
        >
          <SliderTrack :class="sliderTrack">
            <SliderRange :class="sliderRange" />
          </SliderTrack>
          <SliderThumb :class="sliderThumb" aria-label="Position" />
        </SliderRoot>
        <div class="flex justify-between text-xs tabular-nums opacity-60">
          <span>{{ formatTime(currentTime) }}</span>
          <span>{{ formatTime(duration) }}</span>
        </div>
      </div>
    </template>
  </div>
</template>
