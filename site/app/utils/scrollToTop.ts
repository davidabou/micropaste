export function scrollToTop() {
  history.replaceState(null, '', location.pathname)
  window.scrollTo({ top: 0 })
}
