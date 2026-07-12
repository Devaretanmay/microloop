#!/usr/bin/env node

import { type ChildProcess, spawn } from "child_process";
import * as fs from "fs";
import * as path from "path";
/**
 * CLI entry point for the refactored coding agent.
 * Uses main.ts with AgentSession and new mode modules.
 *
 * Test with: npx tsx src/cli-new.ts [args...]
 */
import { APP_NAME } from "./config.ts";
import { configureHttpDispatcher } from "./core/http-dispatcher.ts";
import { main } from "./main.ts";

process.title = APP_NAME;
process.env.MICROLOOP_CODING_AGENT = "true";
process.emitWarning = (() => {}) as typeof process.emitWarning;

import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Spawn the microloop-proxy sidecar
// Proxy stdout goes to a log file to prevent eprintln! diagnostics from
// corrupting the TUI display. The log file is in the project root.
const devProxyPath = path.resolve(__dirname, "../../../../../target/debug/microloop-proxy");
const devLogPath = path.resolve(__dirname, "../../../../../microloop-proxy.log");

const execDir = path.dirname(process.execPath);
const bundledProxyPath = path.resolve(execDir, "microloop-proxy");

const proxyPath = fs.existsSync(bundledProxyPath) ? bundledProxyPath : devProxyPath;
const proxyLogPath = fs.existsSync(bundledProxyPath) ? path.resolve(execDir, "microloop-proxy.log") : devLogPath;
let proxyProcess: ChildProcess | null = null;
try {
	const logFd = fs.openSync(proxyLogPath, "a");
	proxyProcess = spawn(proxyPath, [], {
		stdio: ["ignore", logFd, logFd],
	});
	
} catch (err) {
	console.error(`[Harness] Failed to start proxy sidecar: ${err}`);
	process.exit(1);
}

// Route LLM requests through the proxy sidecar (listening on 8080)
// We use base URL overrides instead of HTTP_PROXY so we don't break npm install
process.env.OPENAI_BASE_URL = "http://127.0.0.1:8080/v1";
process.env.ANTHROPIC_BASE_URL = "http://127.0.0.1:8080";

function cleanupProxy() {
	if (proxyProcess && !proxyProcess.killed) {
		proxyProcess.kill("SIGTERM");
		// Force kill after 3 seconds if it hasn't exited
		setTimeout(() => {
			if (!proxyProcess.killed) {
				proxyProcess.kill("SIGKILL");
			}
		}, 3000).unref();
	}
}

process.on("exit", cleanupProxy);
process.on("SIGINT", () => {
	cleanupProxy();
	process.exit(0);
});
process.on("SIGTERM", cleanupProxy);
process.on("uncaughtException", (err) => {
	console.error(`[Harness] Uncaught error: ${err}`);
	cleanupProxy();
	process.exit(1);
});
process.on("unhandledRejection", (err) => {
	console.error(`[Harness] Unhandled rejection: ${err}`);
	cleanupProxy();
	process.exit(1);
});

// Configure undici's global dispatcher before provider SDKs issue requests.
// Runtime settings are applied once SettingsManager has loaded global/project settings.
configureHttpDispatcher();

main(process.argv.slice(2));
