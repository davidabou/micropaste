/** The link if the copied text is a single http(s) URL, else `null`. */
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

/** Domain shown for a link, without `www.`. */
export function linkDomain(link: string): string {
  return new URL(link).hostname.replace(/^www\./, '')
}
