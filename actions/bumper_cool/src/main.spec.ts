import api from './api'
import { commitForRelease, prepareEdit } from './main'
import { Response } from 'node-fetch'

describe('commitForRelease()', () => {
  it('Should handle fixed commit message', () => {
    expect(
      commitForRelease('This is a fixed commit message', {
        packageName: 'test package',
      })
    ).toBe('This is a fixed commit message')
  })

  it('Should handle template with package name', () => {
    expect(
      commitForRelease('chore({{packageName}}): version {{version}}', {
        packageName: 'test package',
      })
    ).toBe('chore(test package): version {{version}}')
  })

  it('Should handle template with package name and version', () => {
    expect(
      commitForRelease('{packageName} {version}', {
        packageName: 'test package',
        version: 'v1.2.3',
      })
    ).toBe('{packageName} {version}')
  })

  it('Should handle upgrade version template', () => {
    expect(
      commitForRelease(
        'chore({{packageName}}): upgrade to version {{version}}',
        {
          packageName: 'test package',
          version: 'v1.2.3',
        }
      )
    ).toBe('chore(test package): upgrade to version v1.2.3')
  })

  it('Should handle complex template with package name and version', () => {
    expect(
      commitForRelease(
        '{{packageName}} {{version}}: upgrade {{packageName}} to version {{version}}',
        {
          packageName: 'test package',
          version: 'v1.2.3',
        }
      )
    ).toBe('test package v1.2.3: upgrade test package to version v1.2.3')
  })

  it('Should handle constructor and proto', () => {
    expect(commitForRelease('{{constructor}}{{__proto__}}', {})).toBe(
      '{{constructor}}{{__proto__}}'
    )
  })

  it('Should handle template with only version', () => {
    expect(commitForRelease('{{version}}', { version: '{{version}}' })).toBe(
      '{{version}}'
    )
  })
})

describe('prepareEdit()', () => {
  const ctx = {
    sha: 'TAGSHA',
    ref: 'refs/tags/v0.8.2',
    repo: {
      owner: 'OWNER',
      repo: 'REPO',
    },
  }

  process.env['INPUT_PACKAGE-REPO'] = 'xasopheno/weresocool'
  process.env['INPUT_COMMIT-MESSAGE'] = 'Upgrade {{packageName}} to {{version}}'

  // FIXME: this tests results in a live HTTP request. Figure out how to stub the `stream()` method in
  // calculate-download-checksum.
  const stubbedFetch = function (url: string) {
    if (url == 'https://api.github.com/repos/OWNER/REPO/tarball/v0.8.2') {
      return Promise.resolve(
        new Response('', {
          status: 301,
          headers: {
            Location:
              'https://github.com/mislav/bump-homebrew-package-action/archive/v1.9.tar.gz',
          },
        })
      )
    }
    throw url
  }
  const apiClient = api('ATOKEN', { fetch: stubbedFetch, logRequests: false })

  it('should prepare edits correctly', async () => {
    const opts = await prepareEdit(ctx, apiClient, apiClient)
    expect(opts.owner).toBe('xasopheno')
    expect(opts.repo).toBe('weresocool')
    expect(opts.branch).toBe('')
    expect(opts.filePath).toBe('PKGBUILD')
    expect(opts.commitMessage).toBe('Upgrade repo to 0.8.2')

    const oldFormula = `
class MyProgram <package
revision 12
head "git://example.com/repo.git",
revision: "GITSHA"
end
`
    expect(opts.replace(oldFormula)).toBe(`
class MyProgram <package
head "git://example.com/repo.git",
revision: "GITSHA"
end
`)
  })
})
