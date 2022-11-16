"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const ava_1 = __importDefault(require("ava"));
const url_1 = require("url");
const calculateChecksum_1 = require("./calculateChecksum");
ava_1.default('calculate-download-checksum parseArchiveUrl', (t) => {
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
    ];
    tests.forEach((tt) => {
        const archive = calculateChecksum_1.parseArchiveUrl(new url_1.URL(tt.url));
        if (archive == null) {
            t.fail(`did not match: ${tt.url}`);
            return;
        }
        t.is(tt.wants.owner, archive.owner);
        t.is(tt.wants.repo, archive.repo);
        t.is(tt.wants.ref, archive.ref);
        t.is(tt.wants.ext, archive.ext);
    });
});
ava_1.default('calculate-download-checksum parseReleaseDownloadUrl', (t) => {
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
    ];
    tests.forEach((tt) => {
        const asset = calculateChecksum_1.parseReleaseDownloadUrl(new url_1.URL(tt.url));
        if (asset == null) {
            t.fail(`did not match: ${tt.url}`);
            return;
        }
        t.is(tt.wants.owner, asset.owner);
        t.is(tt.wants.repo, asset.repo);
        t.is(tt.wants.tagname, asset.tagname);
        t.is(tt.wants.name, asset.name);
    });
});
//# sourceMappingURL=calculateChecksum.spec.js.map