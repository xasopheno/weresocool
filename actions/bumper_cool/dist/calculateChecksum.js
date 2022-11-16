"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseReleaseDownloadUrl = exports.parseArchiveUrl = void 0;
const core_1 = require("@actions/core");
const url_1 = require("url");
const crypto_1 = require("crypto");
const http_1 = require("http");
const https_1 = require("https");
function stream(url, headers, cb) {
    return new Promise((resolve, reject) => {
        ;
        (url.protocol == 'https:' ? https_1.get : http_1.get)(url, { headers }, (res) => {
            if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400) {
                const loc = res.headers['location'];
                if (loc == null)
                    throw `HTTP ${res.statusCode} but no Location header`;
                const nextURL = new url_1.URL(loc);
                log(nextURL);
                resolve(stream(nextURL, headers, cb));
                return;
            }
            else if (res.statusCode && res.statusCode >= 400) {
                throw new Error(`HTTP ${res.statusCode}`);
            }
            res.on('data', (d) => cb(d));
            res.on('end', () => resolve());
        }).on('error', (err) => reject(err));
    });
}
async function resolveDownload(apiClient, url) {
    if (url.hostname == 'github.com') {
        const api = apiClient.rest;
        const archive = parseArchiveUrl(url);
        if (archive != null) {
            const { owner, repo, ref } = archive;
            const res = await (archive.ext == '.zip'
                ? api.repos.downloadZipballArchive
                : api.repos.downloadTarballArchive)({
                owner,
                repo,
                ref,
                request: {
                    redirect: 'manual',
                },
            });
            const loc = res.headers['location'];
            // HACK: removing "legacy" from the codeload URL ensures that we get the
            // same archive file as web download. Otherwise, the downloaded archive
            // contains resolved commit SHA instead of the tag name in directory path.
            return new url_1.URL(loc.replace('/legacy.', '/'));
        }
        const download = parseReleaseDownloadUrl(url);
        if (download != null) {
            const { owner, repo } = download;
            const tag = download.tagname;
            const res = await api.repos.getReleaseByTag({ owner, repo, tag });
            const asset = res.data.assets.find((a) => a.name == download.name);
            if (asset == null) {
                throw new Error(`could not find asset '${download.name}' in '${tag}' release`);
            }
            const assetRes = await apiClient.request(asset.url, {
                headers: { accept: 'application/octet-stream' },
                request: { redirect: 'manual' },
            });
            const loc = assetRes.headers['location'];
            return new url_1.URL(loc);
        }
    }
    return url;
}
function parseArchiveUrl(url) {
    const match = url.pathname.match(/^\/([^/]+)\/([^/]+)\/archive\/(.+)(\.tar\.gz|\.zip)$/);
    if (match == null) {
        return null;
    }
    return {
        owner: match[1],
        repo: match[2],
        ref: match[3],
        ext: match[4],
    };
}
exports.parseArchiveUrl = parseArchiveUrl;
function parseReleaseDownloadUrl(url) {
    const match = url.pathname.match(/^\/([^/]+)\/([^/]+)\/releases\/download\/(.+)$/);
    if (match == null) {
        return null;
    }
    const parts = match[3].split('/');
    if (parts.length < 2) {
        return null;
    }
    const name = parts.pop() || '';
    return {
        owner: match[1],
        repo: match[2],
        tagname: decodeURIComponent(parts.join('/')),
        name: name,
    };
}
exports.parseReleaseDownloadUrl = parseReleaseDownloadUrl;
function log(url) {
    const params = Array.from(url.searchParams.keys());
    const q = params.length > 0 ? `?${params.join(',')}` : '';
    core_1.debug(`GET ${url.protocol}//${url.hostname}${url.pathname}${q}`);
}
async function default_1(api, url, algorithm) {
    const downloadUrl = await resolveDownload(api, new url_1.URL(url));
    const requestHeaders = { accept: 'application/octet-stream' };
    const hash = crypto_1.createHash(algorithm);
    log(downloadUrl);
    await stream(downloadUrl, requestHeaders, (chunk) => hash.update(chunk));
    return hash.digest('hex');
}
exports.default = default_1;
//# sourceMappingURL=calculateChecksum.js.map