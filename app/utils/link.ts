export function copiedLink(text: string): string | null {
  const link = text.trim()
  if (!/^https?:\/\/\S+$/i.test(link) || !URL.canParse(link)) return null
  return link
}

export function linkDomain(link: string): string {
  return new URL(link).hostname.replace(/^www\./, '')
}
