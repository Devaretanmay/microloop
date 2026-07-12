import { stripVTControlCharacters } from "node:util";
import { describe, expect, it } from "vitest";	describe("stripVTControlCharacters", () => {
	it("throws TypeError for non-string values", () => {
		const $ = stripVTControlCharacters as (value: unknown) => string;

		const expectedMessages: Array<[unknown, string]> = [
			[undefined, "Received undefined"],
			[null, "Received null"],
			[123, "Received type number (123)"],
			[{}, "Received an instance of Object"],
			[Object("x"), "Received an instance of String"],
		];
		for (const [value, suffix] of expectedMessages) {
			expect(() => $(value)).toThrow(TypeError);
			expect(() => $(value)).toThrow(suffix);
		}
	});

	it("strips RIS without leaking the final byte", () => {
		expect(stripVTControlCharacters("\x1bcdone")).toBe("done");
	});

	it("strips single-byte ESC sequences without leaking final bytes", () => {
		for (let code = "g".charCodeAt(0); code <= "m".charCodeAt(0); code++) {
			expect(stripVTControlCharacters(`\x1b${String.fromCharCode(code)}ok`)).toBe("ok");
		}
		for (let code = "r".charCodeAt(0); code <= "t".charCodeAt(0); code++) {
			expect(stripVTControlCharacters(`\x1b${String.fromCharCode(code)}ok`)).toBe("ok");
		}
	});

	it("strips common ANSI sequences used in tool output", () => {
		const input = "a\x1b[31mred\x1b[0m\x1b]8;;https://example.com\x07link\x1b]8;;\x07z";
		expect(stripVTControlCharacters(input)).toBe("aredlinkz");
	});
});
