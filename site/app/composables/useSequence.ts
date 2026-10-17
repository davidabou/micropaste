// Runs a scene script once mounted; its waits never resolve after unmount, which stops it
export function useSequence(script: (wait: (seconds: number) => Promise<void>) => Promise<void>) {
  const timers: ReturnType<typeof setTimeout>[] = []

  function wait(seconds: number) {
    return new Promise<void>(resolve => timers.push(setTimeout(resolve, seconds * 1000)))
  }

  onMounted(() => script(wait))

  onBeforeUnmount(() => timers.forEach(clearTimeout))
}
