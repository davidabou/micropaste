// On several lines: on one, Nuxt's auto-import scanner reads the commas as more exports and skips the next one
export const ease = [
  0.16,
  1,
  0.3,
  1
] as const

export const layoutSpring = {
  type: 'spring' as const,
  bounce: 0.15,
  visualDuration: 0.5
}

export function appear(delay = 0, y = 20) {
  return {
    initial: { opacity: 0, y },
    animate: { opacity: 1, y: 0 },
    transition: { duration: 0.8, ease, delay }
  }
}

export function reveal(delay = 0, amount = 0.5) {
  return {
    initial: { opacity: 0, y: 20 },
    whileInView: { opacity: 1, y: 0 },
    inViewOptions: { once: true, amount },
    transition: { duration: 0.8, ease, delay }
  }
}
