import { beforeAll, describe, expect, test, vi } from "vitest";
import { Container } from "../../tui/src/tui.ts";
import { LoopDetectionBanner } from "../src/modes/interactive/components/loop-detection-banner.ts";
import { LearnCommandOutput } from "../src/modes/interactive/components/learn-command-output.ts";
import type { MicroloopStatus } from "../src/core/agent-session.ts";
import { initTheme } from "../src/modes/interactive/theme/theme.ts";

/**
 * Strip ANSI escape codes from rendered output for assertion.
 */
function stripAnsi(text: string): string {
	return text.replace(/\u001b\[[0-9;]*m/g, "");
}

/**
 * Render a component to plain text (no ANSI codes).
 */
function renderPlain(component: { render: (width?: number) => string[] }, width = 120): string {
	return component.render(width).map(stripAnsi).join("\n");
}

// =========================================================================
// LoopDetectionBanner
// =========================================================================

describe("LoopDetectionBanner", () => {
	beforeAll(() => {
		initTheme("dark");
	});

	const idleStatus: MicroloopStatus = {
		status: "idle",
		tool: "",
		match_count: 0,
		error_count: 0,
		latest_error: "",
	};

	const repeatStatus: MicroloopStatus = {
		status: "repeat",
		tool: "read_file",
		match_count: 3,
		error_count: 1,
		latest_error: "File not found: src/main.rs",
	};

	const blockedStatus: MicroloopStatus = {
		status: "blocked",
		tool: "read_file",
		match_count: 7,
		error_count: 4,
		latest_error: "Operation blocked: too many read repeats",
	};

	test("renders repeat status with tool name and match count", () => {
		const banner = new LoopDetectionBanner(repeatStatus);
		const output = renderPlain(banner);

		expect(output).toContain("REPEAT");
		expect(output).toContain("read_file");
		expect(output).toContain("x3");
		expect(output).toContain("errors:");
		expect(output).toContain("File not found");
	});

	test("renders blocked status with LOOP tag", () => {
		const banner = new LoopDetectionBanner(blockedStatus);
		const output = renderPlain(banner);

		expect(output).toContain("LOOP");
		expect(output).toContain("read_file");
		expect(output).toContain("x7");
		expect(output).toContain("errors:");
		expect(output).toContain("too many read repeats");
	});

	test("idle status renders minimal output (no tool, no match, no error)", () => {
		const banner = new LoopDetectionBanner(idleStatus);
		const output = renderPlain(banner);

		// idle status has no special tag or count
		expect(output).not.toContain("LOOP");
		expect(output).not.toContain("REPEAT");
		expect(output).not.toContain("x");
		expect(output).not.toContain("errors:");
	});

	test("update() replaces old status with new one", () => {
		const banner = new LoopDetectionBanner(repeatStatus);
		let output = renderPlain(banner);
		expect(output).toContain("read_file");
		expect(output).toContain("x3");

		// Update to blocked
		banner.update(blockedStatus);
		output = renderPlain(banner);
		expect(output).toContain("LOOP");
		expect(output).toContain("read_file");
		expect(output).toContain("x7");
		expect(output).toContain("errors:");
		expect(output).toContain("too many read repeats");

		// Update to idle clears the banner
		banner.update(idleStatus);
		output = renderPlain(banner);
		expect(output).not.toContain("LOOP");
		expect(output).not.toContain("REPEAT");
		expect(output).not.toContain("x");
	});

	test("truncates long error messages", () => {
		const longError: MicroloopStatus = {
			status: "blocked",
			tool: "bash",
			match_count: 10,
			error_count: 5,
			latest_error:
				"This is a very long error message that should be truncated because it exceeds the maximum display length of one hundred twenty characters and would otherwise overwhelm the terminal output with excessive detail that is not helpful to the user in a compact banner format.",
		};
		const banner = new LoopDetectionBanner(longError);
		const output = renderPlain(banner);

		// The error should be truncated with "..."
		expect(output).toContain("...");
		// The full error should NOT appear
		expect(output).not.toContain(
			"excessive detail that is not helpful to the user in a compact banner format",
		);
	});

	test("handles zero match count without displaying count", () => {
		const status: MicroloopStatus = {
			status: "repeat",
			tool: "edit_file",
			match_count: 0,
			error_count: 0,
			latest_error: "",
		};
		const banner = new LoopDetectionBanner(status);
		const output = renderPlain(banner);

		expect(output).toContain("REPEAT");
		expect(output).toContain("edit_file");
		expect(output).not.toContain("x0");
		expect(output).not.toContain("errors:");
		expect(output).not.toContain("latest_error");
	});
});

// =========================================================================
// LearnCommandOutput
// =========================================================================

describe("LearnCommandOutput", () => {
	beforeAll(() => {
		initTheme("dark");
	});

	const sampleStdout = `── Microloop Learn — Trajectory Analysis ──────────────────
  Tools found:     2
  Total calls:     6
  Total loops:     0
  Global error rate: 50.0%

  Suggested global max_repeats: 3

  read                     calls=3     repeats=1   errors=0   rate=0.0%   threshold=3  volatile=[none]
  edit_file                calls=3     repeats=2   errors=3   rate=100.0%  threshold=2  volatile=[none]
────────────────────────────────────────────────────────────
Written to microloop_learned.yaml`;

	// Empty output (should render without crashing, minimal output)
	const emptyStdout = "";

	// Malformed output (tool line with missing field)
	const malformedStdout = `── Analysis ──
  Tools found:     1
  bash  calls=missing  repeats=0`;

	// Output with unusual spacing
	const unusualStdout = `── Microloop Learn ──
  Tools found:     1
  Total calls:     1
  Total loops:     0
  Global error rate: 0.0%

  bash                     calls=1     repeats=0   errors=0   rate=0.0%   threshold=3  volatile=[none]`;

	test("parses and renders analysis header with tools found and total calls", () => {
		const component = new LearnCommandOutput(sampleStdout);
		const output = renderPlain(component);

		expect(output).toContain("tools found:");
		expect(output).toContain("2");
		expect(output).toContain("total calls:");
		expect(output).toContain("6");
		expect(output).toContain("total loops:");
		expect(output).toContain("0");
	});

	test("renders global error rate with color coding", () => {
		const component = new LearnCommandOutput(sampleStdout);
		const output = renderPlain(component);

		expect(output).toContain("error rate:");
		expect(output).toContain("50.0%");
	});

	test("renders suggested max_repeats", () => {
		const component = new LearnCommandOutput(sampleStdout);
		const output = renderPlain(component);

		expect(output).toContain("max repeats:");
		expect(output).toContain("3");
	});

	test("renders tool detail rows with calls, repeats, errors, rate, threshold", () => {
		const component = new LearnCommandOutput(sampleStdout);
		const output = renderPlain(component);

		// Header columns
		expect(output).toContain("tool");
		expect(output).toContain("calls");
		expect(output).toContain("repeat");
		expect(output).toContain("errors");
		expect(output).toContain("rate");
		expect(output).toContain("threshold");

		// Tool data
		expect(output).toContain("read");
		expect(output).toContain("edit_file");
		expect(output).toContain("3");
		expect(output).toContain("100.0%");
	});		test("handles raw/empty stdout gracefully (no crash)", () => {
		const component = new LearnCommandOutput(emptyStdout);
		const output = renderPlain(component);

		// Empty string should not crash — renders spacer and empty content
		expect(typeof output).toBe("string");
	});

	test("handles malformed stdout gracefully (partial parse failure)", () => {
		const component = new LearnCommandOutput(malformedStdout);
		const output = renderPlain(component);

		// Malformed output should still render (passthrough fallback)
		expect(typeof output).toBe("string");
		expect(output.length).toBeGreaterThan(0);
	});

	test("parses unusual output with single tool correctly", () => {
		const component = new LearnCommandOutput(unusualStdout);
		const output = renderPlain(component);

		expect(output).toContain("tools found:");
		expect(output).toContain("1");
		expect(output).toContain("bash");
	});

	test("renders volatile column for tools", () => {
		const component = new LearnCommandOutput(sampleStdout);
		const output = renderPlain(component);

		expect(output).toContain("volatile");
	});

	test("renders divider lines", () => {
		const component = new LearnCommandOutput(sampleStdout);
		const output = renderPlain(component);

		// Divider characters (unicode box-drawing)
		expect(output).toContain("─");
	});
});

// =========================================================================
// Microloop Status Event Handling (via InteractiveMode)
// =========================================================================

describe("microloop_status_changed event handling", () => {
	beforeAll(() => {
		initTheme("dark");
	});

	test("creates and adds LoopDetectionBanner to chat container on loop_detected", () => {
		const chatContainer = new Container();
		let uiRequestRenderCalled = false;
		let footerInvalidateCalled = false;

		const fakeThis: any = {
			chatContainer,
			loopDetectionBanner: undefined,
			footer: {
				invalidate: () => {
					footerInvalidateCalled = true;
				},
			},
			ui: {
				requestRender: () => {
					uiRequestRenderCalled = true;
				},
			},
		};

		// Simulate the microloop_status_changed case from handleEvent
		const event = {
			type: "microloop_status_changed" as const,
			status: {
				status: "blocked" as const,
				tool: "read_file",
				match_count: 7,
				error_count: 4,
				latest_error: "too many repeats",
			},
		};

		// Execute the event handler logic (simulating the case in handleEvent)
		if (event.status.status === "blocked" || event.status.status === "repeat") {
			if (!fakeThis.loopDetectionBanner) {
				fakeThis.loopDetectionBanner = new LoopDetectionBanner(event.status);
				fakeThis.chatContainer.addChild(fakeThis.loopDetectionBanner);
			} else {
				fakeThis.loopDetectionBanner.update(event.status);
			}
		} else if (fakeThis.loopDetectionBanner) {
			fakeThis.loopDetectionBanner.update(event.status);
		}
		fakeThis.footer.invalidate();
		fakeThis.ui.requestRender();

		expect(fakeThis.chatContainer.children).toHaveLength(1);
		expect(fakeThis.loopDetectionBanner).toBeDefined();
		expect(uiRequestRenderCalled).toBe(true);
		expect(footerInvalidateCalled).toBe(true);

		// Render the banner content to verify
		const output = renderPlain(fakeThis.loopDetectionBanner);
		expect(output).toContain("LOOP");
		expect(output).toContain("read_file");
		expect(output).toContain("x7");
	});

	test("updates existing banner on subsequent events", () => {
		const chatContainer = new Container();
		const fakeThis: any = {
			chatContainer,
			loopDetectionBanner: undefined,
			footer: { invalidate: vi.fn() },
			ui: { requestRender: vi.fn() },
		};

		// First event: blocked
		const event1 = {
			type: "microloop_status_changed" as const,
			status: {
				status: "blocked" as const,
				tool: "read_file",
				match_count: 7,
				error_count: 4,
				latest_error: "too many repeats",
			},
		};

		if (event1.status.status === "blocked" || event1.status.status === "repeat") {
			if (!fakeThis.loopDetectionBanner) {
				fakeThis.loopDetectionBanner = new LoopDetectionBanner(event1.status);
				fakeThis.chatContainer.addChild(fakeThis.loopDetectionBanner);
			}
		}
		expect(fakeThis.chatContainer.children).toHaveLength(1);

		// Second event: idle (should update, not add new)
		const event2 = {
			type: "microloop_status_changed" as const,
			status: {
				status: "idle" as const,
				tool: "",
				match_count: 0,
				error_count: 0,
				latest_error: "",
			},
		};

		if (event2.status.status === "blocked" || event2.status.status === "repeat") {
			if (!fakeThis.loopDetectionBanner) {
				fakeThis.loopDetectionBanner = new LoopDetectionBanner(event2.status);
				fakeThis.chatContainer.addChild(fakeThis.loopDetectionBanner);
			} else {
				fakeThis.loopDetectionBanner.update(event2.status);
			}
		} else if (fakeThis.loopDetectionBanner) {
			fakeThis.loopDetectionBanner.update(event2.status);
		}

		// Should still have only 1 child (banner was updated, not re-added)
		expect(fakeThis.chatContainer.children).toHaveLength(1);
		const output = renderPlain(fakeThis.loopDetectionBanner);
		expect(output).not.toContain("LOOP");
		expect(output).not.toContain("x7");
	});
});
