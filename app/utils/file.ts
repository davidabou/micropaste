/** Last part of a Windows path, e.g. "photo.png" for "C:\Users\me\photo.png", "C:" for a drive. */
export function fileName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path
}

/** Folder of a Windows path, e.g. "C:\Users\me" for "C:\Users\me\photo.png". */
export function folderOf(path: string): string {
  return path.replace(/[\\/]+$/, '').replace(/[\\/][^\\/]*$/, '')
}
