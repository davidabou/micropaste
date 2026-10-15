const SVG_START = /^\s*(?:<\?xml[^>]*>\s*)?(?:<!--[\s\S]*?-->\s*)*(?:<!DOCTYPE[^>]*>\s*)?<svg[\s>]/i
const SVG_END = /<\/svg>\s*$/i

export function svgToDataUrl(text: string, color: string): string | null {
  if (!SVG_START.test(text) || !SVG_END.test(text)) return null
  if (typeof DOMParser === 'undefined') return null

  // Parsing an XML document never runs its scripts
  const doc = new DOMParser().parseFromString(text, 'image/svg+xml')
  const svg = doc.documentElement
  if (svg.localName !== 'svg' || doc.getElementsByTagName('parsererror').length) return null

  // Required for the SVG to render as an image
  if (!svg.hasAttribute('xmlns')) svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg')
  if (!svg.hasAttribute('color')) svg.setAttribute('color', color)
  if (!svg.hasAttribute('fill')) svg.setAttribute('fill', 'currentColor')

  const markup = new XMLSerializer().serializeToString(svg)
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(markup)}`
}
