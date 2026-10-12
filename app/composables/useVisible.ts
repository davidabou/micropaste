import type { Ref } from 'vue'

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
      { rootMargin: '200px' },
    )
    observer.observe(element.value)
  })

  onBeforeUnmount(() => observer?.disconnect())

  return readonly(visible)
}
