import semver from 'semver'

const RE = /([0-9]+)|([a-zA-Z]+)/g

function parse(ver: string): ReadonlyArray<string | number> {
  const parts = []
  for (const m of ver.matchAll(RE)) {
    parts.push(m[1] ? parseInt(m[1]) : m[2])
  }
  if (parts[0] == 'v') parts.shift()
  return parts
}

export class UpgradeError extends Error {}

export function assertNewer(newVersion: string, oldVersion: string): void {
  if (semver.lt(newVersion, oldVersion)) {
    throw new UpgradeError(
      `the formula version '${oldVersion}' is newer than '${newVersion}'`
    )
  }
  if (!semver.gt(newVersion, oldVersion)) {
    throw new UpgradeError(`the formula is already at version '${newVersion}'`)
  }
}

function escape(value: string, char: string): string {
  return value.replace(new RegExp(`\\${char}`, 'g'), `\\${char}`)
}

export function replaceFields(
  oldContent: string,
  replacements: Map<string, string>
): string {
  let newContent = oldContent
  for (const [field, value] of replacements) {
    newContent = newContent.replace(
      new RegExp(`^(\\s*)${field}(.*?\n)`, 'm'),
      (s: string, indent: string, old: string): string => {
        if (field == 'pkgver=') assertNewer(value, old)
        // else if (field == 'url' && !value.endsWith('.git'))
        // assertNewer(fromUrl(value), fromUrl(old))
        return `${indent}${field}${value}\n`
      }
    )
  }
  return newContent
}

export function removeRevisionLine(oldContent: string): string {
  return oldContent.replace(/^[ \t]*revision \d+ *\r?\n/m, '')
}
