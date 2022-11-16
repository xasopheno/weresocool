"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const ava_1 = __importDefault(require("ava"));
const replace_1 = require("./replace");
ava_1.default('replaceFields() - good version', (t) => {
    const input = `
    pkgname=weresocool
    pkgver=0.9.0
    pkgrel=1
  `;
    const expected = `
    pkgname=weresocool
    pkgver=0.11.1
    pkgrel=1
  `;
    const replacements = new Map();
    replacements.set('pkgver=', '0.11.1');
    t.is(replace_1.replaceFields(input, replacements), expected);
});
ava_1.default('replaceFields() - old version', (t) => {
    const input = `
    pkgname=weresocool
    pkgver=0.12.0
    pkgrel=1
  `;
    const replacements = new Map();
    replacements.set('pkgver=', '0.11.1');
    t.throws(() => replace_1.replaceFields(input, replacements));
});
ava_1.default('replaceFields() - same version', (t) => {
    const input = `
    pkgname=weresocool
    pkgver=0.11.0
    pkgrel=1
  `;
    const replacements = new Map();
    replacements.set('pkgver=', '0.11.0');
    t.throws(() => replace_1.replaceFields(input, replacements));
});
//# sourceMappingURL=replace.spec.js.map