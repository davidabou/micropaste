import { describe, expect, it } from 'vitest'
import { copiedLink, linkDomain, linkLabel } from '@/utils/link'

describe('copiedLink', () => {
  it('keeps web links, trimmed', () => {
    expect(copiedLink('  https://example.com/a?b=1\n')).toBe('https://example.com/a?b=1')
    expect(copiedLink('HTTP://example.com')).toBe('HTTP://example.com')
  })

  it.each(['example.com', 'ftp://example.com', 'https://', 'see https://example.com', 'https://exa mple.com', ''])(
    'rejects %j',
    text => expect(copiedLink(text)).toBeNull()
  )
})

describe('linkLabel', () => {
  it('drops the scheme, www and a trailing slash', () => {
    expect(linkLabel('https://www.example.com/')).toBe('example.com')
    expect(linkLabel('HTTP://torph.lochie.me/blog/post?a=1')).toBe('torph.lochie.me/blog/post?a=1')
    expect(linkLabel('https://example.com/a/')).toBe('example.com/a')
  })
})

describe('linkDomain', () => {
  it('drops www', () => {
    expect(linkDomain('https://www.example.com/page')).toBe('example.com')
    expect(linkDomain('https://docs.example.com')).toBe('docs.example.com')
  })
})
