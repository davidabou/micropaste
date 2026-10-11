import { execSync } from 'node:child_process'
import { defineConfig } from 'bumpp'

// The release commit takes every change (see `all`): work in progress would end up in it
if (execSync('git status --porcelain').toString().trim()) {
  throw new Error('Commit or stash your changes before releasing')
}

// `bun run release`: asks for the new version, writes it everywhere, then commits and tags it.
// Pushing the tag builds the installer into a draft GitHub release (see release.yml)
export default defineConfig({
  commit: 'chore: release v%s',
  tag: true,
  push: false,
  // Cargo.lock holds the app version too, and CI checks it with --locked: let Cargo update it
  execute: 'cargo update --workspace --manifest-path src-tauri/Cargo.toml',
  // Commits Cargo.lock along with the bumped files
  all: true,
  files: ['package.json', 'src-tauri/tauri.conf.json', 'src-tauri/Cargo.toml']
})
