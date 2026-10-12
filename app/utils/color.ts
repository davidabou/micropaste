export interface CopiedColor {
  value: string
  css: string
  format: string
}

const HEX = /^#(?:[0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i
const BARE_HEX = /^(?:[0-9a-f]{6}|[0-9a-f]{8})$/i
const COLOR_FUNCTION = /^(rgba?|hsla?|hwb|lab|lch|oklab|oklch|color)\(\s*[^()]*\)$/i

export function copiedColor(text: string): CopiedColor | null {
  const value = text.trim()
  if (HEX.test(value)) return { value, css: value, format: 'HEX' }
  if (BARE_HEX.test(value)) return { value, css: `#${value}`, format: 'HEX' }

  const match = COLOR_FUNCTION.exec(value)
  if (!match || typeof CSS === 'undefined' || !CSS.supports('color', value)) return null
  return { value, css: value, format: match[1]!.toLowerCase().replace(/^(rgb|hsl)a$/, '$1').toUpperCase() }
}

let pixel: CanvasRenderingContext2D | null = null

export function isLightColor(css: string): boolean {
  pixel ??= document.createElement('canvas').getContext('2d', { willReadFrequently: true })
  if (!pixel) return true
  pixel.clearRect(0, 0, 1, 1)
  pixel.fillStyle = css
  pixel.fillRect(0, 0, 1, 1)
  const [r, g, b, a] = pixel.getImageData(0, 0, 1, 1).data as unknown as [number, number, number, number]
  const backdrop = 230
  const luminance = [r, g, b]
    .map(channel => (channel * a + backdrop * (255 - a)) / 255 / 255)
    .map(c => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4))
    .reduce((sum, c, i) => sum + c * [0.2126, 0.7152, 0.0722][i]!, 0)
  // Contrast with black (L + 0.05) / 0.05 against white 1.05 / (L + 0.05): equal at L ≈ 0.179
  return luminance > 0.179
}
