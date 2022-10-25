import test from 'ava'
import api from './api'
import { commitForRelease, prepareEdit } from './main'
import { Response } from 'node-fetch'

test('commitForRelease()', (t) => {
  t.is(
    commitForRelease('This is a fixed commit message', {
      packageName: 'test package',
    }),
    'This is a fixed commit message'
  )
  t.is(
    commitForRelease('chore({{packageName}}): version {{version}}', {
      packageName: 'test package',
    }),
    'chore(test package): version {{version}}'
  )
  t.is(
    commitForRelease('{packageName} {version}', {
      packageName: 'test package',
      version: 'v1.2.3',
    }),
    '{packageName} {version}'
  )
  t.is(
    commitForRelease('chore({{packageName}}): upgrade to version {{version}}', {
      packageName: 'test package',
      version: 'v1.2.3',
    }),
    'chore(test package): upgrade to version v1.2.3'
  )
  t.is(
    commitForRelease(
      '{{packageName}} {{version}}: upgrade {{packageName}} to version {{version}}',
      {
        packageName: 'test package',
        version: 'v1.2.3',
      }
    ),
    'test package v1.2.3: upgrade test package to version v1.2.3'
  )
  t.is(
    commitForRelease('{{constructor}}{{__proto__}}', {}),
    '{{constructor}}{{__proto__}}'
  )
  t.is(
    commitForRelease('{{version}}', { version: '{{version}}' }),
    '{{version}}'
  )
})

test('prepareEdit()', async (t) => {
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

  const opts = await prepareEdit(ctx, apiClient, apiClient)
  t.is(opts.owner, 'xasopheno')
  t.is(opts.repo, 'weresocool')
  t.is(opts.branch, '')
  t.is(opts.filePath, 'PKGBUILD')
  t.is(opts.commitMessage, 'Upgrade repo to 0.8.2')

  const oldFormula = `
    class MyProgram <package 
      revision 12
      head "git://example.com/repo.git",
        revision: "GITSHA"
    end
  `
  t.is(
    `
    class MyProgram <package 
      head "git://example.com/repo.git",
        revision: "GITSHA"
    end
  `,
    opts.replace(oldFormula)
  )
})
