import type { ComponentPublicInstance, Ref } from 'vue'
import { animate } from 'motion-v'

type Target = Element | ComponentPublicInstance | null | undefined

// Stays on its target every frame, so it follows it while layout animations move it
export function usePointer(stage: Ref<HTMLElement | undefined>) {
  const x = useMotionValue(0)
  const y = useMotionValue(0)
  const progress = useMotionValue(1)

  let from = { x: 0, y: 0 }
  let target: { el: Target, at: [number, number] } | { x: number, y: number } | null = null

  function point() {
    const root = stage.value
    if (!root) return null

    if (!target) return { x: root.offsetWidth * 0.85, y: root.offsetHeight * 1.1 }
    if (!('el' in target)) return target

    const el = target.el && '$el' in target.el ? target.el.$el as Element : target.el
    if (!el) return null

    const box = root.getBoundingClientRect()
    const scale = box.width / root.offsetWidth || 1
    const rect = el.getBoundingClientRect()
    return {
      x: (rect.left - box.left + rect.width * target.at[0]) / scale,
      y: (rect.top - box.top + rect.height * target.at[1]) / scale
    }
  }

  useAnimationFrame(() => {
    const to = point()
    if (!to) return
    const k = progress.get()
    x.set(from.x + (to.x - from.x) * k)
    y.set(from.y + (to.y - from.y) * k)
  })

  async function moveTo(el: Target, at: [number, number] = [0.5, 0.5], duration = 0.6) {
    from = { x: x.get(), y: y.get() }
    target = { el, at }
    progress.jump(0)
    await animate(progress, 1, { duration, ease: [0.65, 0, 0.35, 1] })
  }

  // Stops following the target, e.g. while it moves away after the action
  function hold() {
    target = { x: x.get(), y: y.get() }
    from = target
    progress.jump(1)
  }

  return { x, y, moveTo, hold }
}
