"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.removeRevisionLine = exports.replaceFields = exports.assertNewer = exports.UpgradeError = void 0;
const semver_1 = __importDefault(require("semver"));
const RE = /([0-9]+)|([a-zA-Z]+)/g;
function parse(ver) {
    const parts = [];
    for (const m of ver.matchAll(RE)) {
        parts.push(m[1] ? parseInt(m[1]) : m[2]);
    }
    if (parts[0] == 'v')
        parts.shift();
    return parts;
}
class UpgradeError extends Error {
}
exports.UpgradeError = UpgradeError;
function assertNewer(newVersion, oldVersion) {
    if (semver_1.default.lt(newVersion, oldVersion)) {
        throw new UpgradeError(`the formula version '${oldVersion}' is newer than '${newVersion}'`);
    }
    if (!semver_1.default.gt(newVersion, oldVersion)) {
        throw new UpgradeError(`the formula is already at version '${newVersion}'`);
    }
}
exports.assertNewer = assertNewer;
function escape(value, char) {
    return value.replace(new RegExp(`\\${char}`, 'g'), `\\${char}`);
}
function replaceFields(oldContent, replacements) {
    let newContent = oldContent;
    for (const [field, value] of replacements) {
        newContent = newContent.replace(new RegExp(`^(\\s*)${field}(.*?\n)`, 'm'), (s, indent, old) => {
            if (field == 'pkgver=')
                assertNewer(value, old);
            // else if (field == 'url' && !value.endsWith('.git'))
            // assertNewer(fromUrl(value), fromUrl(old))
            return `${indent}${field}${value}\n`;
        });
    }
    return newContent;
}
exports.replaceFields = replaceFields;
function removeRevisionLine(oldContent) {
    return oldContent.replace(/^[ \t]*revision \d+ *\r?\n/m, '');
}
exports.removeRevisionLine = removeRevisionLine;
//# sourceMappingURL=replace.js.map