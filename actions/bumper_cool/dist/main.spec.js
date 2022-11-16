"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const ava_1 = __importDefault(require("ava"));
const api_1 = __importDefault(require("./api"));
const main_1 = require("./main");
const node_fetch_1 = require("node-fetch");
ava_1.default('commitForRelease()', (t) => {
    t.is(main_1.commitForRelease('This is a fixed commit message', {
        packageName: 'test package',
    }), 'This is a fixed commit message');
    t.is(main_1.commitForRelease('chore({{packageName}}): version {{version}}', {
        packageName: 'test package',
    }), 'chore(test package): version {{version}}');
    t.is(main_1.commitForRelease('{packageName} {version}', {
        packageName: 'test package',
        version: 'v1.2.3',
    }), '{packageName} {version}');
    t.is(main_1.commitForRelease('chore({{packageName}}): upgrade to version {{version}}', {
        packageName: 'test package',
        version: 'v1.2.3',
    }), 'chore(test package): upgrade to version v1.2.3');
    t.is(main_1.commitForRelease('{{packageName}} {{version}}: upgrade {{packageName}} to version {{version}}', {
        packageName: 'test package',
        version: 'v1.2.3',
    }), 'test package v1.2.3: upgrade test package to version v1.2.3');
    t.is(main_1.commitForRelease('{{constructor}}{{__proto__}}', {}), '{{constructor}}{{__proto__}}');
    t.is(main_1.commitForRelease('{{version}}', { version: '{{version}}' }), '{{version}}');
});
ava_1.default('prepareEdit()', async (t) => {
    const ctx = {
        sha: 'TAGSHA',
        ref: 'refs/tags/v0.8.2',
        repo: {
            owner: 'OWNER',
            repo: 'REPO',
        },
    };
    process.env['INPUT_PACKAGE-REPO'] = 'xasopheno/weresocool';
    process.env['INPUT_COMMIT-MESSAGE'] = 'Upgrade {{packageName}} to {{version}}';
    // FIXME: this tests results in a live HTTP request. Figure out how to stub the `stream()` method in
    // calculate-download-checksum.
    const stubbedFetch = function (url) {
        if (url == 'https://api.github.com/repos/OWNER/REPO/tarball/v0.8.2') {
            return Promise.resolve(new node_fetch_1.Response('', {
                status: 301,
                headers: {
                    Location: 'https://github.com/mislav/bump-homebrew-package-action/archive/v1.9.tar.gz',
                },
            }));
        }
        throw url;
    };
    const apiClient = api_1.default('ATOKEN', { fetch: stubbedFetch, logRequests: false });
    const opts = await main_1.prepareEdit(ctx, apiClient, apiClient);
    t.is(opts.owner, 'xasopheno');
    t.is(opts.repo, 'weresocool');
    t.is(opts.branch, '');
    t.is(opts.filePath, 'PKGBUILD');
    t.is(opts.commitMessage, 'Upgrade repo to 0.8.2');
    const oldFormula = `
    class MyProgram <package 
      revision 12
      head "git://example.com/repo.git",
        revision: "GITSHA"
    end
  `;
    t.is(`
    class MyProgram <package 
      head "git://example.com/repo.git",
        revision: "GITSHA"
    end
  `, opts.replace(oldFormula));
});
//# sourceMappingURL=main.spec.js.map