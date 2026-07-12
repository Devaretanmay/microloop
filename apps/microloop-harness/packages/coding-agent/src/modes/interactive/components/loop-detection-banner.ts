import { Container, Spacer, Text } from "@microloop/tui";
import type { MicroloopStatus } from "../../../core/agent-session.ts";
import { theme } from "../theme/theme.ts";

/**
 * Banner component displayed in the chat area when a microloop loop or
 * repeat pattern is detected by the proxy. Shows the tool name, match
 * count, error rate, and the latest error message when applicable.
 */
export class LoopDetectionBanner extends Container {
	private status: MicroloopStatus;

	constructor(status: MicroloopStatus) {
		super();
		this.status = status;
		this.addChild(new Spacer(1));
		this.addChild(new Text(this.renderBanner(), 1, 0));
	}

	update(status: MicroloopStatus): void {
		this.status = status;
		this.clear();
		this.addChild(new Spacer(1));
		this.addChild(new Text(this.renderBanner(), 1, 0));
	}

	private renderBanner(): string {
		const isBlocked = this.status.status === "blocked";
		const isRepeat = this.status.status === "repeat";
		const isActive = isBlocked || isRepeat;

		if (!isActive) {
			return "";
		}

		// Border
		const borderColor = isBlocked ? theme.fg("error", "\u2500") : theme.fg("warning", "\u2500");
		const border = borderColor.repeat(4);

		// Status label
		const statusTag = isBlocked
			? theme.bold(theme.fg("error", " LOOP "))
			: theme.bold(theme.fg("warning", " REPEAT "));

		// Tool name
		const toolLabel =
			this.status.tool
				? ` ${theme.fg("text", theme.bold(this.status.tool))}`
				: "";

		// Match count
		const matchInfo =
			this.status.match_count > 0
				? ` ${theme.fg("dim", "x")}${theme.fg("text", String(this.status.match_count))}`
				: "";

		// Error count
		const errorInfo =
			this.status.error_count > 0
				? ` ${theme.fg("error", "errors:")}${theme.fg("text", String(this.status.error_count))}`
				: "";

		// Latest error (truncated)
		const errorDetail =
			this.status.latest_error
				? `\n  ${theme.fg("dim", this.truncateError(this.status.latest_error, 120))}`
				: "";

		// Build the full banner
		return [
			`${border}[${statusTag}${toolLabel}${matchInfo}${errorInfo} ]${border}`,
			errorDetail,
		].filter((l) => l).join("\n");
	}

	private truncateError(text: string, maxLen: number): string {
		return text.length <= maxLen ? text : text.slice(0, maxLen) + "...";
	}
}
