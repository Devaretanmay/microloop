import { execFile } from "child_process";
import { existsSync } from "fs";
import { dirname, isAbsolute, resolve } from "path";
import { fileURLToPath } from "url";
import type { AgentTool } from "@microloop/agent";
import { type Static, Type } from "typebox";
import type { ToolDefinition } from "../extensions/types.ts";
import { wrapToolDefinition } from "./tool-definition-wrapper.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const learnSchema = Type.Object({
	input: Type.String({ description: "Path to a session .jsonl file to analyze" }),
	summarize: Type.Optional(Type.Boolean({ description: "Print a human-readable summary alongside the YAML policy" })),
});

export type LearnToolInput = Static<typeof learnSchema>;

/**
 * Resolve the microloop-learn binary path relative to this file.
 * Source: apps/microloop-harness/packages/coding-agent/src/core/tools/learn.ts
 * Target: <project_root>/target/<profile>/microloop-learn (7 levels up)
 */
function resolveLearnBinary(): string {
	const releasePath = resolve(__dirname, "../../../../../../../target/release/microloop-learn");
	const debugPath = resolve(__dirname, "../../../../../../../target/debug/microloop-learn");

	if (existsSync(releasePath)) {
		return releasePath;
	}
	if (existsSync(debugPath)) {
		return debugPath;
	}
	throw new Error(
		`Could not find microloop-learn binary. Looked in:\n  ${releasePath}\n  ${debugPath}\n\nBuild it with: cargo build -p microloop-learn --release`,
	);
}

export function createLearnToolDefinition(
	cwd: string,
): ToolDefinition<typeof learnSchema, undefined> {
	return {
		name: "learn",
		label: "learn",
		description:
			"Analyze an exported session .jsonl file via microloop-learn to detect loops and generate a safety policy. " +
			"The output is a YAML policy with per-tool repeat thresholds. Use summarize=true to also get a human-readable summary.",
		promptSnippet: "Analyze session .jsonl files for loop detection",
		promptGuidelines: [
			"Use this when the user asks to analyze a session file for loop detection or generate a safety guardrail policy",
			"The user must provide a path to an exported .jsonl session file",
			"Output is always a YAML policy; set summarize=true for an additional human-readable summary table",
		],
		parameters: learnSchema,
		async execute(
			_toolCallId,
			{ input, summarize }: { input: string; summarize?: boolean },
			signal?: AbortSignal,
			_onUpdate?,
			_ctx?,
		) {
			if (signal?.aborted) {
				throw new Error("Operation aborted");
			}

			// Validate input path exists
			const resolvedInputPath = isAbsolute(input) ? input : resolve(cwd, input);
			if (!existsSync(resolvedInputPath)) {
				throw new Error(`Input file not found: ${resolvedInputPath}`);
			}

			const binaryPath = resolveLearnBinary();
			const args: string[] = ["--input", resolvedInputPath];
			if (summarize) {
				args.push("--summarize");
			}

			return new Promise((resolve, reject) => {
				const child = execFile(binaryPath, args, { maxBuffer: 10 * 1024 * 1024 }, (error, stdout, stderr) => {
					if (error) {
						if (error.code === "ENOENT") {
							reject(new Error(`microloop-learn binary not found at ${binaryPath}`));
						} else if (stderr) {
							reject(new Error(stderr.trim()));
						} else {
							reject(new Error(`microloop-learn exited with code ${error.code ?? "unknown"}`));
						}
						return;
					}

					resolve({
						content: [{ type: "text", text: stdout.trim() }],
						details: undefined,
					});
				});

				// Cancel the running process on abort
				if (signal) {
					signal.addEventListener("abort", () => {
						child.kill();
						reject(new Error("Operation aborted"));
					}, { once: true });
				}
			});
		},
	};
}

export function createLearnTool(cwd: string): AgentTool<typeof learnSchema> {
	return wrapToolDefinition(createLearnToolDefinition(cwd));
}
