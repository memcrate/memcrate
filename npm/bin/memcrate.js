#!/usr/bin/env node
// Thin shim: forwards every argument to the real binary fetched by postinstall
// and exits with its exit code.

const fs = require('fs')
const path = require('path')
const { spawnSync } = require('child_process')

const bin = process.platform === 'win32' ? 'memcrate.exe' : 'memcrate'
const target = path.join(__dirname, '..', 'vendor', bin)

if (!fs.existsSync(target)) {
  console.error('memcrate: binary not found.')
  console.error(
    'The install step that downloads it did not run. This usually means the\n' +
      'package was installed with --ignore-scripts. Re-run:\n' +
      '  npm rebuild memcrate\n' +
      'or install another way: cargo install memcrate'
  )
  process.exit(1)
}

const result = spawnSync(target, process.argv.slice(2), { stdio: 'inherit' })

if (result.error) {
  console.error(`memcrate: failed to run binary: ${result.error.message}`)
  process.exit(1)
}

process.exit(result.status === null ? 1 : result.status)
