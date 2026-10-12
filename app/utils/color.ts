export interface CopiedColor {
  /** The color as copied, e.g. "#ff5733" or "F54927" */
  value: string
  /** The same color usable in CSS */
  css: string
  /** Notation shown on the tile, e.g. "HEX", "RGB", "OKLCH" */
  format: string
}

const HEX = /^#(?:[0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i
const BARE_HEX = /^(?:[0-9a-f]{6}|[0-9a-f]{8})$/i
const COLOR_FUNCTION = /^(rgba?|hsla?|hwb|lab|lch|oklab|oklch|color)\(\s*[^()]*\)$/i

/**
 * The color if the copied text is a single CSS color: hex (`#ff5733`, or `F54927` without `#`)
 * or color function (`rgb()`, `hsl()`, `oklch()`…). Color names like "red" stay text, they are usually just words.
 */
export function copiedColor(text: string): CopiedColor | null {
  const value = text.trim()
  if (HEX.test(value)) return { value, css: value, format: 'HEX' }
  // Without `#`, as design tools copy it
  if (BARE_HEX.test(value)) return { value, css: `#${value}`, format: 'HEX' }

  const match = COLOR_FUNCTION.exec(value)
  // Browser APIs only: no preview during server rendering
  if (!match || typeof CSS === 'undefined' || !CSS.supports('color', value)) return null
  // rgba() and hsla() are the same notations as rgb() and hsl()
  return { value, css: value, format: match[1]!.toLowerCase().replace(/^(rgb|hsl)a$/, '$1').toUpperCase() }
}

let pixel: CanvasRenderingContext2D | null = null

/**
 * Whether dark text reads better than white text on a color: the one with the higher WCAG contrast.
 * The browser draws the color on a 1px canvas, so any CSS notation works (oklch(), lab()…).
 * A see-through color is taken over the light checkerboard drawn behind it.
 */
export function isLightColor(css: string): boolean {
  pixel ??= document.createElement('canvas').getContext('2d', { willReadFrequently: true })
  if (!pixel) return true
  pixel.clearRect(0, 0, 1, 1)
  pixel.fillStyle = css
  pixel.fillRect(0, 0, 1, 1)
  const [r, g, b, a] = pixel.getImageData(0, 0, 1, 1).data as unknown as [number, number, number, number]
  // Average gray of the checkerboard (#ccc and #fff)
  const backdrop = 230
  const luminance = [r, g, b]
    .map(channel => (channel * a + backdrop * (255 - a)) / 255 / 255)
    .map(c => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4))
    .reduce((sum, c, i) => sum + c * [0.2126, 0.7152, 0.0722][i]!, 0)
  // Contrast with black (L + 0.05) / 0.05 against white 1.05 / (L + 0.05): equal at L ≈ 0.179
  return luminance > 0.179
}
