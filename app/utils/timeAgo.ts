import type { FormatTimeAgoOptions } from '@vueuse/core'

const MINUTE = 60_000
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR

export const compactTimeAgo: FormatTimeAgoOptions = {
  rounding: 'floor',
  units: [
    { max: HOUR, value: MINUTE, name: 'minute' },
    { max: DAY, value: HOUR, name: 'hour' },
    { max: 7 * DAY, value: DAY, name: 'day' },
    { max: 30 * DAY, value: 7 * DAY, name: 'week' },
    { max: 365 * DAY, value: 30 * DAY, name: 'month' },
    { max: Infinity, value: 365 * DAY, name: 'year' }
  ],
  messages: {
    justNow: 'now',
    past: n => (/\d/.test(n) ? `${n} ago` : n),
    future: () => 'now',
    second: n => `${n} s`,
    minute: n => `${n} min`,
    hour: n => `${n} hr`,
    day: n => `${n} d`,
    week: n => `${n} wk`,
    month: n => `${n} mo`,
    year: n => `${n} yr`,
    invalid: ''
  }
}
