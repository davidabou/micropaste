const SVG_START = /^\s*(?:<\?xml[^>]*>\s*)?(?:<!--[\s\S]*?-->\s*)*(?:<!DOCTYPE[^>]*>\s*)?<svg[\s>]/i
const SVG_END = /<\/svg>\s*$/i

function parseSvg(text: string): Element | null {
  if (!SVG_START.test(text) || !SVG_END.test(text)) return null
  if (typeof DOMParser === 'undefined') return null

  // Parsing an XML document never runs its scripts
  const doc = new DOMParser().parseFromString(text, 'image/svg+xml')
  const svg = doc.documentElement
  if (svg.localName !== 'svg' || doc.getElementsByTagName('parsererror').length) return null
  return svg
}

export function svgToDataUrl(text: string, color: string): string | null {
  const svg = parseSvg(text)
  if (!svg) return null

  // Required for the SVG to render as an image
  if (!svg.hasAttribute('xmlns')) svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg')
  if (!svg.hasAttribute('color')) svg.setAttribute('color', color)
  if (!svg.hasAttribute('fill')) svg.setAttribute('fill', 'currentColor')

  const markup = new XMLSerializer().serializeToString(svg)
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(markup)}`
}

const LENGTH = /^\s*(\d+(?:\.\d+)?|\.\d+)(?:px)?\s*$/i

export function svgSize(text: string): { width: number, height: number } | null {
  const svg = parseSvg(text)
  if (!svg) return null
  const length = (name: string) => Number(LENGTH.exec(svg.getAttribute(name) ?? '')?.[1] ?? Number.NaN)
  const width = length('width')
  const height = length('height')
  if (width > 0 && height > 0) return { width, height }
  const box = (svg.getAttribute('viewBox') ?? '').trim().split(/[\s,]+/).map(Number)
  if (box.length === 4 && box[2]! > 0 && box[3]! > 0) return { width: box[2]!, height: box[3]! }
  return null
}
