import { formatTimeAgo } from '@vueuse/core'
import { describe, expect, it } from 'vitest'
import { compactTimeAgo } from '@/utils/timeAgo'

const MINUTE = 60_000
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR
const now = new Date('2026-01-01T12:00:00Z')

describe('compactTimeAgo', () => {
  it.each([
    [10_000, 'now'],
    [MINUTE, '1 min ago'],
    [59 * MINUTE, '59 min ago'],
    [90 * MINUTE, '1 hr ago'],
    [DAY, '1 d ago'],
    [8 * DAY, '1 wk ago'],
    [45 * DAY, '1 mo ago'],
    [400 * DAY, '1 yr ago'],
    [-HOUR, 'now']
  ])('%d ms ago → %s', (ago, text) => {
    expect(formatTimeAgo(new Date(now.getTime() - ago), compactTimeAgo, now)).toBe(text)
  })
})
