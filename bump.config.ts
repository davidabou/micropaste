import { execSync } from 'node:child_process'
import { defineConfig } from 'bumpp'

if (execSync('git status --porcelain').toString().trim()) {
  throw new Error('Commit or stash your changes before releasing')
}

export default defineConfig({
  commit: 'chore: release v%s',
  tag: true,
  push: false,
  // Cargo.lock update
  execute: 'cargo update --workspace --manifest-path src-tauri/Cargo.toml',
  all: true,
  files: ['package.json', 'src-tauri/tauri.conf.json', 'src-tauri/Cargo.toml']
})
