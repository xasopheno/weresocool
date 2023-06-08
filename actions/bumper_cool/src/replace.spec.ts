import { replaceFields } from './replace'

describe('replaceFields', () => {
  it('replaces fields correctly - good version', () => {
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

    expect(replaceFields(input, replacements)).toEqual(expected)
  })

  it('throws error for replacing fields - old version', () => {
    const input = `
      pkgname=weresocool
      pkgver=0.12.0
      pkgrel=1
    `

    const replacements = new Map<string, string>()
    replacements.set('pkgver=', '0.11.1')

    expect(() => replaceFields(input, replacements)).toThrow()
  })

  it('throws error for replacing fields - same version', () => {
    const input = `
      pkgname=weresocool
      pkgver=0.11.0
      pkgrel=1
    `

    const replacements = new Map<string, string>()
    replacements.set('pkgver=', '0.11.0')

    expect(() => replaceFields(input, replacements)).toThrow()
  })
})
