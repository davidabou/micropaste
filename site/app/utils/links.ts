import { version } from '../../../package.json'

export const SITE_URL = 'https://micropaste.abou.dev'
export const REPO_URL = 'https://github.com/davidabou/micropaste'
// The app version in the root package.json, so the buttons download the installer directly
export const DOWNLOAD_URL = `${REPO_URL}/releases/download/v${version}/Micropaste_${version}_x64-setup.exe`
export const ISSUES_URL = `${REPO_URL}/issues`
export const SPONSOR_URL = 'https://github.com/sponsors/davidabou'

export const sections = [
  { label: 'Features', to: '#features' },
  { label: 'Collections', to: '#collections' },
  { label: 'Windows 11', to: '#windows' },
  { label: 'FAQ', to: '#faq' }
]
