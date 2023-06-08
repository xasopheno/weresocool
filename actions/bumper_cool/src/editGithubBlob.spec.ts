import api from './api'
import { Response } from 'node-fetch'
import editGithubBlob from './editGithubBlob'

type fetchOptions = {
  method: string
  body: string | null
}

function replyJSON(status: number, body: unknown): Promise<Response> {
  return Promise.resolve(
    new Response(JSON.stringify(body), {
      status,
      headers: {
        'Content-Type': 'application/json',
      },
    })
  )
}

describe('editGithubBlob', () => {
  test('direct push', async () => {
    const stubbedFetch = function (url: string, options: fetchOptions) {
      function route(method: string, path: string): boolean {
        return (
          method.toUpperCase() === options.method.toUpperCase() &&
          `https://api.github.com/${path}` === url
        )
      }

      if (route('GET', 'repos/OWNER/REPO')) {
        return replyJSON(200, {
          default_branch: 'main',
          permissions: { push: true },
        })
      } else if (route('GET', 'repos/OWNER/REPO/branches/main')) {
        return replyJSON(200, {
          commit: { sha: 'COMMITSHA' },
          protected: false,
        })
      } else if (route('GET', 'repos/OWNER/REPO/contents/PKGBUILD?ref=main')) {
        return replyJSON(200, {
          content: Buffer.from(`old content`).toString('base64'),
        })
      } else if (route('PUT', 'repos/OWNER/REPO/contents/PKGBUILD')) {
        const payload = JSON.parse(options.body || '')
        expect('main').toBe(payload.branch)
        expect('Update PKGBUILD').toBe(payload.message)
        expect(Buffer.from(payload.content, 'base64').toString()).toBe(
          'OLD CONTENT'
        )
        return replyJSON(200, {
          commit: { html_url: 'https://github.com/OWNER/REPO/commit/NEWSHA' },
        })
      }
      throw `not stubbed: ${options.method} ${url}`
    }

    const url = await editGithubBlob({
      apiClient: api('ATOKEN', { fetch: stubbedFetch, logRequests: false }),
      owner: 'OWNER',
      repo: 'REPO',
      filePath: 'PKGBUILD',
      replace: (oldContent) => oldContent.toUpperCase(),
    })
    expect(url).toBe('https://github.com/OWNER/REPO/commit/NEWSHA')
  })

  test('via pull request', async () => {
    let newBranchName: string
    const stubbedFetch = function (url: string, options: fetchOptions) {
      function route(method: string, path: string): boolean {
        return (
          method.toUpperCase() === options.method.toUpperCase() &&
          `https://api.github.com/${path}` === url
        )
      }

      if (route('GET', 'repos/OWNER/REPO')) {
        return replyJSON(200, {
          default_branch: 'main',
          permissions: { push: false },
        })
      } else if (route('GET', 'repos/OWNER/REPO/branches/main')) {
        return replyJSON(200, {
          commit: { sha: 'COMMITSHA' },
          protected: false,
        })
      } else if (route('POST', 'repos/OWNER/REPO/forks')) {
        return replyJSON(200, {})
      } else if (route('GET', 'user')) {
        return replyJSON(200, { login: 'FORKOWNER' })
      } else if (route('POST', 'repos/FORKOWNER/REPO/merge-upstream')) {
        const payload = JSON.parse(options.body || '')
        expect('main').toBe(payload.branch)
        return replyJSON(409, {})
      } else if (route('POST', 'repos/FORKOWNER/REPO/git/refs')) {
        const payload = JSON.parse(options.body || '')
        expect(payload.ref).toMatch(/^refs\/heads\/update-PKGBUILD-\d+$/)
        newBranchName = payload.ref.replace('refs/heads/', '')
        expect('COMMITSHA').toBe(payload.sha)
        return replyJSON(201, {})
      } else if (
        route(
          'GET',
          `repos/FORKOWNER/REPO/contents/PKGBUILD?ref=${newBranchName}`
        )
      ) {
        return replyJSON(200, {
          content: Buffer.from(`old content`).toString('base64'),
        })
      } else if (route('PUT', 'repos/FORKOWNER/REPO/contents/PKGBUILD')) {
        const payload = JSON.parse(options.body || '')
        expect(newBranchName).toBe(payload.branch)
        expect('Update PKGBUILD').toBe(payload.message)
        expect(Buffer.from(payload.content, 'base64').toString()).toBe(
          'OLD CONTENT'
        )
        return replyJSON(200, {
          commit: { html_url: 'https://github.com/OWNER/REPO/commit/NEWSHA' },
        })
      } else if (route('POST', 'repos/OWNER/REPO/pulls')) {
        const payload = JSON.parse(options.body || '')
        expect('main').toBe(payload.base)
        expect(`FORKOWNER:${newBranchName}`).toBe(payload.head)
        expect('Update PKGBUILD').toBe(payload.title)
        expect('').toBe(payload.body)
        return replyJSON(201, {
          html_url: 'https://github.com/OWNER/REPO/pull/123',
        })
      }
      throw `not stubbed: ${options.method} ${url}`
    }

    const url = await editGithubBlob({
      apiClient: api('ATOKEN', { fetch: stubbedFetch, logRequests: false }),
      owner: 'OWNER',
      repo: 'REPO',
      filePath: 'PKGBUILD',
      replace: (oldContent) => oldContent.toUpperCase(),
    })
    expect(url).toBe('https://github.com/OWNER/REPO/pull/123')
  })

  test('with pushTo', async () => {
    let newBranchName: string
    const stubbedFetch = function (url: string, options: fetchOptions) {
      function route(method: string, path: string): boolean {
        return (
          method.toUpperCase() === options.method.toUpperCase() &&
          `https://api.github.com/${path}` === url
        )
      }

      if (route('GET', 'repos/OWNER/REPO')) {
        return replyJSON(200, {
          default_branch: 'main',
          permissions: { push: false },
        })
      } else if (route('GET', 'repos/OWNER/REPO/branches/main')) {
        return replyJSON(200, {
          commit: { sha: 'COMMITSHA' },
          protected: false,
        })
      } else if (route('POST', 'repos/FORKOWNER/REPO/merge-upstream')) {
        const payload = JSON.parse(options.body || '')
        expect('main').toBe(payload.branch)
        return replyJSON(409, {})
      } else if (route('POST', 'repos/FORKOWNER/REPO/git/refs')) {
        const payload = JSON.parse(options.body || '')
        expect(payload.ref).toMatch(/^refs\/heads\/update-PKGBUILD-\d+$/)
        newBranchName = payload.ref.replace('refs/heads/', '')
        expect('COMMITSHA').toBe(payload.sha)
        return replyJSON(201, {})
      } else if (
        route(
          'GET',
          `repos/FORKOWNER/REPO/contents/PKGBUILD?ref=${newBranchName}`
        )
      ) {
        return replyJSON(200, {
          content: Buffer.from(`old content`).toString('base64'),
        })
      } else if (route('PUT', 'repos/FORKOWNER/REPO/contents/PKGBUILD')) {
        const payload = JSON.parse(options.body || '')
        expect(newBranchName).toBe(payload.branch)
        expect('Update PKGBUILD').toBe(payload.message)
        expect(Buffer.from(payload.content, 'base64').toString()).toBe(
          'OLD CONTENT'
        )
        return replyJSON(200, {
          commit: { html_url: 'https://github.com/OWNER/REPO/commit/NEWSHA' },
        })
      } else if (route('POST', 'repos/OWNER/REPO/pulls')) {
        const payload = JSON.parse(options.body || '')
        expect('main').toBe(payload.base)
        expect(`FORKOWNER:${newBranchName}`).toBe(payload.head)
        expect('Update PKGBUILD').toBe(payload.title)
        expect('').toBe(payload.body)
        return replyJSON(201, {
          html_url: 'https://github.com/OWNER/REPO/commit/NEWSHA',
        })
      }
      throw `not stubbed: ${options.method} ${url}`
    }

    const url = await editGithubBlob({
      apiClient: api('ATOKEN', { fetch: stubbedFetch, logRequests: false }),
      owner: 'OWNER',
      repo: 'REPO',
      filePath: 'PKGBUILD',
      pushTo: { owner: 'FORKOWNER', repo: 'REPO' },
      replace: (oldContent) => oldContent.toUpperCase(),
    })
    expect(url).toBe('https://github.com/OWNER/REPO/commit/NEWSHA')
  })
})
