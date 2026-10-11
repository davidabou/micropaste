import type { Ref } from 'vue'

/** Becomes `true` the first time the element comes near the visible area, and stays `true`. */
export function useVisible(element: Readonly<Ref<HTMLElement | null>>) {
  const visible = ref(false)
  let observer: IntersectionObserver | undefined

  onMounted(() => {
    if (!element.value) return
    observer = new IntersectionObserver(
      (entries) => {
        if (entries.some(entry => entry.isIntersecting)) {
          visible.value = true
          observer?.disconnect()
        }
      },
      // Start a bit before the element scrolls into view
      { rootMargin: '200px' },
    )
    observer.observe(element.value)
  })

  onBeforeUnmount(() => observer?.disconnect())

  return readonly(visible)
}
