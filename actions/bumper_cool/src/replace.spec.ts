import test from 'ava'
import { replaceFields } from './replace'

test('replaceFields() - good version', (t) => {
  const input = `
    pkgname=weresocool
    pkgver=0.9.0
    pkgrel=1
  `
  const expected = `
    pkgname=weresocool
    pkgver=0.11.1
    pkgrel=1
  `

  const replacements = new Map<string, string>()
  replacements.set('pkgver=', '0.11.1')

  t.is(replaceFields(input, replacements), expected)
})

test('replaceFields() - old version', (t) => {
  const input = `
    pkgname=weresocool
    pkgver=0.12.0
    pkgrel=1
  `

  const replacements = new Map<string, string>()
  replacements.set('pkgver=', '0.11.1')

  t.throws(() => replaceFields(input, replacements))
})

test('replaceFields() - same version', (t) => {
  const input = `
    pkgname=weresocool
    pkgver=0.11.0
    pkgrel=1
  `

  const replacements = new Map<string, string>()
  replacements.set('pkgver=', '0.11.0')

  t.throws(() => replaceFields(input, replacements))
})
