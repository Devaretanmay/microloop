import { Container, Spacer, Text } from "@microloop/tui";
import { theme } from "../theme/theme.ts";

/**
 * Parsed trajectory analysis from the microloop-learn binary output.
 * The binary emits structured table-format output that this component
 * parses and re-renders with proper theme-aware colors.
 */
interface LearnAnalysis {
	toolsFound: number;
	totalCalls: number;
	totalLoops: number;
	globalErrorRate: number;
	suggestedMaxRepeats: number;
	tools: Array<{
		name: string;
		calls: number;
		repeats: number;
		errors: number;
		errorRate: number;
		threshold: number;
		volatile: string;
	}>;
}

/**
 * Component that renders the output of the /learn command in a structured,
 * theme-aware layout with column-aligned tool stats.
 */
export class LearnCommandOutput extends Container {
	constructor(stdout: string) {
		super();
		this.addChild(new Spacer(1));
		this.addChild(new Text(this.renderOutput(stdout), 1, 0));
	}

	private renderOutput(stdout: string): string {
		const analysis = this.parseAnalysis(stdout);
		if (!analysis) {
			// Pass through raw output if parsing fails
			return stdout;
		}

		const lines: string[] = [];

		// Header
		const headerLine = theme.fg("dim", "\u2500".repeat(4)) +
			theme.bold(theme.fg("accent", " Analysis ")) +
			theme.fg("dim", "\u2500".repeat(80));
		lines.push(headerLine);

		// Summary section
		lines.push(
			`  ${theme.fg("dim", "tools found:")} ${theme.fg("text", String(analysis.toolsFound))}`,
		);
		lines.push(
			`  ${theme.fg("dim", "total calls:")} ${theme.fg("text", String(analysis.totalCalls))}`,
		);
		lines.push(
			`  ${theme.fg("dim", "total loops:")} ${theme.fg("text", String(analysis.totalLoops))}`,
		);

		// Error rate with color coding
		const errorRateColor =
			analysis.globalErrorRate >= 50 ? "error"
			: analysis.globalErrorRate >= 20 ? "warning"
			: "success";
		lines.push(
			`  ${theme.fg("dim", "error rate:")} ${theme.fg(errorRateColor, `${analysis.globalErrorRate.toFixed(1)}%`)}`,
		);

		// Suggested max_repeats
		lines.push(
			`  ${theme.fg("dim", "max repeats:")} ${theme.fg("accent", String(analysis.suggestedMaxRepeats))}`,
		);

		lines.push("");

		// Tool detail section
		if (analysis.tools.length > 0) {
			// Table header
			const header = [
				theme.fg("dim", padRight("tool", 20)),
				theme.fg("dim", padRight("calls", 8)),
				theme.fg("dim", padRight("repeat", 8)),
				theme.fg("dim", padRight("errors", 8)),
				theme.fg("dim", padRight("rate", 8)),
				theme.fg("dim", padRight("threshold", 10)),
				theme.fg("dim", "volatile"),
			].join("");

			lines.push(`  ${theme.fg("dim", header)}`);
			lines.push(`  ${theme.fg("dim", "\u2500".repeat(Math.min(80, header.length)))}`);

			for (const tool of analysis.tools) {
				const rateColor =
					tool.errorRate >= 50 ? "error"
					: tool.errorRate >= 20 ? "warning"
					: "success";

				const row = [
					theme.fg("text", padRight(tool.name, 20)),
					theme.fg("text", padRight(String(tool.calls), 8)),
					tool.repeats > 0
						? theme.fg("warning", padRight(String(tool.repeats), 8))
						: theme.fg("dim", padRight(String(tool.repeats), 8)),
					tool.errors > 0
						? theme.fg("error", padRight(String(tool.errors), 8))
						: theme.fg("dim", padRight(String(tool.errors), 8)),
					theme.fg(rateColor, padRight(`${tool.errorRate.toFixed(1)}%`, 8)),
					theme.fg("accent", padRight(String(tool.threshold), 10)),
					tool.volatile === "none"
						? theme.fg("dim", tool.volatile)
						: theme.fg("warning", tool.volatile),
				].join("");

				lines.push(`  ${row}`);
			}
		}

		// Footer
		lines.push(theme.fg("dim", "\u2500".repeat(Math.min(80, headerLine.length))));

		return lines.join("\n");
	}

	private parseAnalysis(raw: string): LearnAnalysis | null {
		try {
			const lines = raw.split("\n").map((l) => l.trim()).filter((l) => l);

			let toolsFound = 0;
			let totalCalls = 0;
			let totalLoops = 0;
			let globalErrorRate = 0;
			let suggestedMaxRepeats = 0;
			const tools: LearnAnalysis["tools"] = [];

			for (let i = 0; i < lines.length; i++) {
				const line = lines[i];

				// Match "Tools found: N"
				const toolsMatch = line.match(/tools found:\s*(\d+)/i);
				if (toolsMatch) {
					toolsFound = parseInt(toolsMatch[1], 10);
					continue;
				}

				// Match "Total calls: N"
				const callsMatch = line.match(/total calls:\s*(\d+)/i);
				if (callsMatch) {
					totalCalls = parseInt(callsMatch[1], 10);
					continue;
				}

				// Match "Total loops: N"
				const loopsMatch = line.match(/total loops:\s*(\d+)/i);
				if (loopsMatch) {
					totalLoops = parseInt(loopsMatch[1], 10);
					continue;
				}

				// Match "Global error rate: X%"
				const errRateMatch = line.match(/global error rate:\s*([\d.]+)%/i);
				if (errRateMatch) {
					globalErrorRate = parseFloat(errRateMatch[1]);
					continue;
				}

				// Match "Suggested global max_repeats: N"
				const maxRepMatch = line.match(/suggested global max_repeats:\s*(\d+)/i);
				if (maxRepMatch) {
					suggestedMaxRepeats = parseInt(maxRepMatch[1], 10);
					continue;
				}

				// Match tool lines like:
				// bash        calls=4     repeats=0   errors=0   rate=0.0%   threshold=3  volatile=[none]
				const toolMatch = line.match(
					/^(\S+)\s+calls=(\d+)\s+repeats=(\d+)\s+errors=(\d+)\s+rate=([\d.]+)%\s+threshold=(\d+)\s+volatile=\[(.+)\]$/i,
				);
				if (toolMatch) {
					tools.push({
						name: toolMatch[1],
						calls: parseInt(toolMatch[2], 10),
						repeats: parseInt(toolMatch[3], 10),
						errors: parseInt(toolMatch[4], 10),
						errorRate: parseFloat(toolMatch[5]),
						threshold: parseInt(toolMatch[6], 10),
						volatile: toolMatch[7],
					});
				}
			}

			return {
				toolsFound,
				totalCalls,
				totalLoops,
				globalErrorRate,
				suggestedMaxRepeats,
				tools,
			};
		} catch {
			return null;
		}
	}
}

function padRight(text: string, width: number): string {
	if (text.length >= width) return text.slice(0, width);
	return text + " ".repeat(width - text.length);
}
