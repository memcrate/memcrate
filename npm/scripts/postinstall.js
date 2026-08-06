#!/usr/bin/env node
// Downloads the memcrate binary matching this package's version from GitHub
// Releases, verifies its published sha256, and unpacks it next to the shim.
//
// Uses only Node built-ins plus `tar` (present on macOS, Linux, and Windows 10+)
// so installing memcrate never pulls in a dependency tree.

const fs = require('fs')
const os = require('os')
const path = require('path')
const crypto = require('crypto')
const { execFileSync } = require('child_process')

const REPO = 'memcrate/memcrate'
const { version } = require('../package.json')
const vendorDir = path.join(__dirname, '..', 'vendor')

const TARGETS = {
  'linux-x64': { target: 'x86_64-unknown-linux-gnu', ext: 'tar.gz', bin: 'memcrate' },
  'darwin-arm64': { target: 'aarch64-apple-darwin', ext: 'tar.gz', bin: 'memcrate' },
  'win32-x64': { target: 'x86_64-pc-windows-msvc', ext: 'zip', bin: 'memcrate.exe' },
}

function fail(message, hint) {
  console.error(`memcrate: ${message}`)
  if (hint) console.error(hint)
  process.exit(1)
}

function resolveTarget() {
  const key = `${process.platform}-${process.arch}`
  const found = TARGETS[key]
  if (!found) {
    fail(
      `no prebuilt binary for ${key}.`,
      'Install from source instead:\n  cargo install memcrate\n' +
        'See https://memcrate.dev for other options.'
    )
  }
  return found
}

async function download(url) {
  const res = await fetch(url, { redirect: 'follow' })
  if (!res.ok) {
    fail(
      `download failed (${res.status}) for ${url}`,
      `Check that release v${version} exists at https://github.com/${REPO}/releases`
    )
  }
  return Buffer.from(await res.arrayBuffer())
}

function extract(archivePath, ext, destDir) {
  if (ext === 'tar.gz') {
    execFileSync('tar', ['-xzf', archivePath, '-C', destDir], { stdio: 'inherit' })
    return
  }
  // Windows: tar.exe ships with Windows 10+ and reads zips; fall back to PowerShell.
  try {
    execFileSync('tar', ['-xf', archivePath, '-C', destDir], { stdio: 'inherit' })
  } catch {
    execFileSync(
      'powershell',
      ['-NoProfile', '-Command', `Expand-Archive -Path "${archivePath}" -DestinationPath "${destDir}" -Force`],
      { stdio: 'inherit' }
    )
  }
}

async function main() {
  const { target, ext, bin } = resolveTarget()
  const asset = `memcrate-${target}.${ext}`
  const base = `https://github.com/${REPO}/releases/download/v${version}/${asset}`

  fs.mkdirSync(vendorDir, { recursive: true })
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'memcrate-'))

  try {
    console.log(`memcrate: downloading ${asset} (v${version})`)
    const archive = await download(base)

    const expected = (await download(`${base}.sha256`)).toString('utf8').trim().split(/\s+/)[0]
    const actual = crypto.createHash('sha256').update(archive).digest('hex')
    if (!expected || expected.toLowerCase() !== actual) {
      fail(`checksum mismatch for ${asset}: expected ${expected}, got ${actual}. Refusing to install.`)
    }

    const archivePath = path.join(tmp, asset)
    fs.writeFileSync(archivePath, archive)
    extract(archivePath, ext, tmp)

    const extracted = path.join(tmp, bin)
    if (!fs.existsSync(extracted)) {
      fail(`archive did not contain ${bin}. Contents: ${fs.readdirSync(tmp).join(', ')}`)
    }

    const dest = path.join(vendorDir, bin)
    fs.copyFileSync(extracted, dest)
    fs.chmodSync(dest, 0o755)
    console.log(`memcrate: installed ${dest}`)
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true })
  }
}

main().catch((err) => fail(err && err.message ? err.message : String(err)))
