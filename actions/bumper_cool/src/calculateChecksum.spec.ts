import { URL } from 'url'
import { parseArchiveUrl, parseReleaseDownloadUrl } from './calculateChecksum'

describe('calculate-download-checksum', () => {
  describe('parseArchiveUrl', () => {
    it('should correctly parse the URL', () => {
      const tests = [
        {
          url: 'https://github.com/xasopheno/weresocool/archive/refs/tags/v2.13.0.tar.gz',
          wants: {
            owner: 'xasopheno',
            repo: 'weresocool',
            ref: 'refs/tags/v2.13.0',
            ext: '.tar.gz',
          },
        },
      ]

      tests.forEach((tt) => {
        const archive = parseArchiveUrl(new URL(tt.url))
        expect(archive).not.toBeNull()
        expect(archive).toMatchObject(tt.wants)
      })
    })
  })

  describe('parseReleaseDownloadUrl', () => {
    it('should correctly parse the URL', () => {
      const tests = [
        {
          url: 'https://github.com/john-u/smartthings-cli/releases/download/%40smartthings%2Fcli%401.0.0-beta.9/smartthings-macos.tar.gz',
          wants: {
            owner: 'john-u',
            repo: 'smartthings-cli',
            tagname: '@smartthings/cli@1.0.0-beta.9',
            name: 'smartthings-macos.tar.gz',
          },
        },
        {
          url: 'https://github.com/john-u/smartthings-cli/releases/download/@smartthings/cli@1.0.0-beta.9/smartthings-macos.tar.gz',
          wants: {
            owner: 'john-u',
            repo: 'smartthings-cli',
            tagname: '@smartthings/cli@1.0.0-beta.9',
            name: 'smartthings-macos.tar.gz',
          },
        },
      ]

      tests.forEach((tt) => {
        const asset = parseReleaseDownloadUrl(new URL(tt.url))
        expect(asset).not.toBeNull()
        expect(asset).toMatchObject(tt.wants)
      })
    })
  })
})
