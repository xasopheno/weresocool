"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.prepareEdit = exports.commitForRelease = void 0;
const core_1 = require("@actions/core");
const editGithubBlob_1 = __importDefault(require("./editGithubBlob"));
const replace_1 = require("./replace");
const github_1 = require("@actions/github");
function tarballForRelease(owner, repo, tagName) {
    return `https://github.com/${owner}/${repo}/archive/${tagName}.tar.gz`;
}
function commitForRelease(messageTemplate, params = {}) {
    return messageTemplate.replace(/\{\{(\w+)\}\}/g, (m, key) => {
        if (Object.hasOwnProperty.call(params, key)) {
            return params[key];
        }
        return m;
    });
}
exports.commitForRelease = commitForRelease;
async function default_1(api) {
    const internalToken = process.env.GITHUB_TOKEN || process.env.COMMITTER_TOKEN || '';
    const externalToken = process.env.COMMITTER_TOKEN || '';
    const options = await prepareEdit(github_1.context, api(internalToken), api(externalToken));
    const createdUrl = await editGithubBlob_1.default(options);
    console.log(createdUrl);
}
exports.default = default_1;
async function prepareEdit(ctx, sameRepoClient, crossRepoClient) {
    const tagName = core_1.getInput('tag-name') ||
        ((ref) => {
            if (!ref.startsWith('refs/tags/'))
                throw `invalid ref: ${ref}`;
            return ref.replace('refs/tags/', '');
        })(ctx.ref);
    const [owner, repo] = core_1.getInput('package-repo', { required: true }).split('/');
    let pushTo;
    const pushToSpec = core_1.getInput('push-to');
    if (pushToSpec) {
        const [pushToOwner, pushToRepo] = pushToSpec.split('/');
        pushTo = { owner: pushToOwner, repo: pushToRepo };
    }
    const packageName = core_1.getInput('package-name') || ctx.repo.repo.toLowerCase();
    const branch = core_1.getInput('base-branch');
    const filePath = core_1.getInput('pkgbuild-path') || `PKGBUILD`;
    console.log('filePath:', filePath);
    const version = tagName.replace(/^v(\d)/, '$1');
    console.log('version', version);
    const downloadUrl = core_1.getInput('download-url') ||
        tarballForRelease(ctx.repo.owner, ctx.repo.repo, tagName);
    const messageTemplate = core_1.getInput('commit-message', { required: true });
    let makePR;
    if (core_1.getInput('create-pullrequest')) {
        makePR = core_1.getBooleanInput('create-pullrequest');
    }
    const replacements = new Map();
    replacements.set('pkgver=', version);
    console.log('replacements:', replacements);
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
    });
    return {
        apiClient: crossRepoClient,
        owner,
        repo,
        branch,
        filePath,
        commitMessage,
        pushTo,
        makePR,
        replace(oldContent) {
            return replace_1.removeRevisionLine(replace_1.replaceFields(oldContent, replacements));
        },
    };
}
exports.prepareEdit = prepareEdit;
//# sourceMappingURL=main.js.map