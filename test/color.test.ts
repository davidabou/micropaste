import { describe, expect, it } from 'vitest'
import { copiedColor } from '@/utils/color'

describe('copiedColor', () => {
  it.each(['#fff', '#FFFA', '#336699', '#33669980'])('reads %s', (hex) => {
    expect(copiedColor(hex)).toEqual({ value: hex, css: hex, format: 'HEX' })
  })

  it('adds # to bare hex of 6 or 8 digits', () => {
    expect(copiedColor(' 336699 ')).toEqual({ value: '336699', css: '#336699', format: 'HEX' })
    expect(copiedColor('fff')).toBeNull()
    expect(copiedColor('123456789')).toBeNull()
  })

  it('names color functions without their alpha suffix', () => {
    expect(copiedColor('rgba(0, 0, 0, 0.5)')?.format).toBe('RGB')
    expect(copiedColor('hsl(120 50% 50%)')?.format).toBe('HSL')
    expect(copiedColor('oklch(0.7 0.1 200)')?.format).toBe('OKLCH')
  })

  it.each(['#ggg', 'rgb(0, 0, 0) and more', 'calc(1px)', 'red', 'some text'])('rejects %j', (text) => {
    expect(copiedColor(text)).toBeNull()
  })
})
