import { getInput, getBooleanInput } from '@actions/core'
import type { API } from './api'
import editGitHubBlob from './editGithubBlob'
import { Options as EditOptions } from './editGithubBlob'
import { removeRevisionLine, replaceFields } from './replace'
import calculateDownloadChecksum from './calculateChecksum'
import { context } from '@actions/github'

function tarballForRelease(
  owner: string,
  repo: string,
  tagName: string
): string {
  return `https://github.com/${owner}/${repo}/archive/${tagName}.tar.gz`
}

export function commitForRelease(
  messageTemplate: string,
  params: { [key: string]: string } = {}
): string {
  return messageTemplate.replace(
    /\{\{(\w+)\}\}/g,
    (m: string, key: string): string => {
      if (Object.hasOwnProperty.call(params, key)) {
        return params[key]
      }
      return m
    }
  )
}

export default async function (api: (token: string) => API): Promise<void> {
  const internalToken =
    process.env.GITHUB_TOKEN || process.env.COMMITTER_TOKEN || ''
  const externalToken = process.env.COMMITTER_TOKEN || ''

  const options = await prepareEdit(
    context,
    api(internalToken),
    api(externalToken)
  )
  const createdUrl = await editGitHubBlob(options)
  console.log(createdUrl)
}

type Context = {
  ref: string
  sha: string
  repo: {
    owner: string
    repo: string
  }
}

export async function prepareEdit(
  ctx: Context,
  sameRepoClient: API,
  crossRepoClient: API
): Promise<EditOptions> {
  const tagName =
    getInput('tag-name') ||
    ((ref) => {
      if (!ref.startsWith('refs/tags/')) throw `invalid ref: ${ref}`
      return ref.replace('refs/tags/', '')
    })(ctx.ref)

  const [owner, repo] = getInput('package-repo', { required: true }).split('/')
  let pushTo: { owner: string; repo: string } | undefined
  const pushToSpec = getInput('push-to')
  if (pushToSpec) {
    const [pushToOwner, pushToRepo] = pushToSpec.split('/')
    pushTo = { owner: pushToOwner, repo: pushToRepo }
  }
  const packageName = getInput('package-name') || ctx.repo.repo.toLowerCase()
  const branch = getInput('base-branch')
  const filePath = getInput('pkgbuild-path') || `PKGBUILD`
  console.log('filePath:', filePath)
  const version = tagName.replace(/^v(\d)/, '$1')
  console.log('version', version)
  const downloadUrl =
    getInput('download-url') ||
    tarballForRelease(ctx.repo.owner, ctx.repo.repo, tagName)
  const messageTemplate = getInput('commit-message', { required: true })

  let makePR: boolean | undefined
  if (getInput('create-pullrequest')) {
    makePR = getBooleanInput('create-pullrequest')
  }

  const replacements = new Map<string, string>()
  replacements.set('pkgver=', version)
  console.log('replacements:', replacements)
  // replacements.set('url', downloadUrl)
  // if (downloadUrl.endsWith('.git')) {
  // replacements.set('tag', tagName)
  // replacements.set(
  // 'revision',
  // await (async () => {
  // if (ctx.ref == `refs/tags/${tagName}`) return ctx.sha
  // else {
  // const res = await sameRepoClient.rest.git.getRef({
  // ...ctx.repo,
  // ref: `tags/${tagName}`,
  // })
  // return res.data.object.sha
  // }
  // })()
  // )
  // } else {
  // replacements.set(
  // 'sha256',
  // getInput('download-sha256') ||
  // (await calculateDownloadChecksum(sameRepoClient, downloadUrl, 'sha256'))
  // )
  // }

  const commitMessage = commitForRelease(messageTemplate, {
    packageName,
    version,
  })

  return {
    apiClient: crossRepoClient,
    owner,
    repo,
    branch,
    filePath,
    commitMessage,
    pushTo,
    makePR,
    replace(oldContent: string) {
      return removeRevisionLine(replaceFields(oldContent, replacements))
    },
  }
}
