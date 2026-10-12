export function copiedLink(text: string): string | null {
  const link = text.trim()
  if (!/^https?:\/\/\S+$/i.test(link)) return null
  try {
    new URL(link)
    return link
  }
  catch {
    return null
  }
}

export function linkDomain(link: string): string {
  return new URL(link).hostname.replace(/^www\./, '')
}
