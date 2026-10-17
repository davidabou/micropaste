export type Clip
  = | { kind: 'link', title: string, domain: string, image: string, app: string, time: string }
    | { kind: 'text', content: string, app: string, time: string }
    | { kind: 'color', value: string, format: string, app: string, time: string }
    | { kind: 'image', src: string, size: string, app: string, time: string }
    | { kind: 'files', icons: string[], app: string, time: string }
    | { kind: 'svg', path: string, app: string, time: string }

export const BOLT = 'M13.5 2 4 13.5h6.5L9 22l10-12h-6.5z'

export const FILE_ICONS = ['vscode-icons:file-type-pdf2', 'vscode-icons:file-type-word', 'vscode-icons:file-type-excel']

export const sampleClips = {
  link: {
    kind: 'link',
    title: 'Nuxt: The Full-Stack Vue Framework',
    domain: 'nuxt.com',
    image: '/nuxt.jpg',
    app: 'logos:chrome',
    time: '2 min ago'
  },
  text: {
    kind: 'text',
    content: 'export function useClips() {\n  const clips = ref<Clip[]>([])\n  const favorites = computed(() =>\n    clips.value.filter(c => c.favorite))\n  return { clips, favorites }\n}',
    app: 'logos:visual-studio-code',
    time: '5 min ago'
  },
  color: {
    kind: 'color',
    value: '#5B5BD6',
    format: 'HEX',
    app: 'logos:figma',
    time: '9 min ago'
  },
  files: {
    kind: 'files',
    icons: FILE_ICONS,
    app: 'flat-color-icons:folder',
    time: '14 min ago'
  },
  image: {
    kind: 'image',
    src: '/background.jpg',
    size: '2.4 MB',
    app: 'logos:microsoft-edge',
    time: '21 min ago'
  },
  svg: {
    kind: 'svg',
    path: BOLT,
    app: 'logos:figma',
    time: '32 min ago'
  }
} satisfies Record<string, Clip>
