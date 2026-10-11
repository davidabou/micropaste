// Shared by every component, follows the Windows theme live
const prefersDark = ref(false)
let watching = false

/** Whether Windows uses the dark theme. */
export function usePrefersDark() {
  if (import.meta.client && !watching) {
    watching = true
    const query = window.matchMedia('(prefers-color-scheme: dark)')
    prefersDark.value = query.matches
    query.addEventListener('change', event => (prefersDark.value = event.matches))
  }
  return readonly(prefersDark)
}
