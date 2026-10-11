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
