"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const core_1 = require("@actions/core");
const core_2 = require("@octokit/core");
const plugin_rest_endpoint_methods_1 = require("@octokit/plugin-rest-endpoint-methods");
const plugin_request_log_1 = require("@octokit/plugin-request-log");
const GitHub = core_2.Octokit.plugin(plugin_rest_endpoint_methods_1.restEndpointMethods, plugin_request_log_1.requestLog).defaults({
    baseUrl: 'https://api.github.com',
});
function default_1(token, options) {
    return new GitHub({
        request: { fetch: options && options.fetch },
        auth: `token ${token}`,
        log: {
            info(msg) {
                if (options && options.logRequests === false)
                    return;
                return console.info(msg);
            },
            debug(msg) {
                if (!core_1.isDebug())
                    return;
                return console.debug(msg);
            },
            warn(msg) {
                return console.warn(msg);
            },
            error(msg) {
                return console.error(msg);
            },
        },
    });
}
exports.default = default_1;
//# sourceMappingURL=api.js.map