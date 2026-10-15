import { describe, expect, it } from 'vitest'
import { formatBytes } from '@/utils/bytes'

describe('formatBytes', () => {
  it.each([
    [0, '0 B'],
    [1023, '1,023 B'],
    [1024, '1 KB'],
    [1536, '1.5 KB'],
    [15 * 1024 + 512, '16 KB'],
    [5 * 1024 ** 2, '5 MB'],
    [3.25 * 1024 ** 3, '3.3 GB'],
    [2048 * 1024 ** 3, '2,048 GB']
  ])('%d → %s', (bytes, text) => {
    expect(formatBytes(bytes)).toBe(text)
  })
})
