import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export const useUpdate = createSharedComposable(() => {
  const currentVersion = ref<string | null>(null)
  const readyVersion = ref<string | null>(null)
  const status = ref<'idle' | 'checking' | 'upToDate' | 'failed'>('idle')

  getVersion().then(version => (currentVersion.value = version))
  invoke<string | null>('ready_update').then(version => (readyVersion.value = version))
  const unlisten = listen<string>('update-ready', ({ payload }) => (readyVersion.value = payload))
  tryOnScopeDispose(() => unlisten.then(stop => stop()))

  async function check() {
    status.value = 'checking'
    try {
      readyVersion.value = await invoke<string | null>('check_for_update')
      status.value = readyVersion.value ? 'idle' : 'upToDate'
    } catch {
      status.value = 'failed'
    }
  }

  const install = () => invoke('install_update')

  return { currentVersion, readyVersion, status, check, install }
})
