import { describe, expect, it } from 'vitest'
import { svgToDataUrl } from '@/utils/svg'

function markup(url: string | null) {
  expect(url).toMatch(/^data:image\/svg\+xml;charset=utf-8,/)
  return decodeURIComponent(url!.split(',')[1]!)
}

describe('svgToDataUrl', () => {
  it('adds the namespace, color and fill an image needs', () => {
    const svg = markup(svgToDataUrl('<svg viewBox="0 0 1 1"><path d="M0 0h1v1z"/></svg>', '#123456'))
    expect(svg).toContain('xmlns="http://www.w3.org/2000/svg"')
    expect(svg).toContain('color="#123456"')
    expect(svg).toContain('fill="currentColor"')
  })

  it('keeps the colors of the SVG', () => {
    const svg = markup(svgToDataUrl('<svg xmlns="http://www.w3.org/2000/svg" fill="red" color="blue"></svg>', '#000'))
    expect(svg).toContain('fill="red"')
    expect(svg).toContain('color="blue"')
  })

  it('accepts an XML prolog, comments and a doctype', () => {
    const text = '<?xml version="1.0"?>\n<!-- icon -->\n<!DOCTYPE svg>\n<svg></svg>\n'
    expect(svgToDataUrl(text, '#000')).not.toBeNull()
  })

  it.each([
    'Here is an icon: <svg></svg>',
    '<svg></svg> and some text',
    '<svgfoo></svgfoo>',
    '<div><svg></svg></div>',
    '<svg><unclosed></svg>'
  ])('rejects %j', (text) => {
    expect(svgToDataUrl(text, '#000')).toBeNull()
  })
})
