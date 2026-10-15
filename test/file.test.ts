import { describe, expect, it } from 'vitest'
import { fileName, folderOf } from '@/utils/file'

describe('fileName', () => {
  it.each([
    ['C:\\Users\\me\\photo.png', 'photo.png'],
    ['C:\\Users\\me\\Documents\\', 'Documents'],
    ['/home/me/notes.txt', 'notes.txt'],
    ['notes.txt', 'notes.txt']
  ])('%s → %s', (path, name) => {
    expect(fileName(path)).toBe(name)
  })
})

describe('folderOf', () => {
  it.each([
    ['C:\\Users\\me\\photo.png', 'C:\\Users\\me'],
    ['C:\\Users\\me\\Documents\\', 'C:\\Users\\me'],
    ['/home/me/notes.txt', '/home/me']
  ])('%s → %s', (path, folder) => {
    expect(folderOf(path)).toBe(folder)
  })
})
