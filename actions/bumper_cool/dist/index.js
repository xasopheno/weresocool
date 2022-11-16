"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const core_1 = require("@actions/core");
const api_1 = __importDefault(require("./api"));
const replace_1 = require("./replace");
const main_1 = __importDefault(require("./main"));
main_1.default(api_1.default).catch((error) => {
    if (error instanceof replace_1.UpgradeError) {
        console.warn('Skipping: %s', error.message);
        return;
    }
    core_1.setFailed(error.toString());
    if (process.env.GITHUB_ACTIONS == undefined) {
        console.error(error.stack);
    }
});
//# sourceMappingURL=index.js.map