import { describe, expect, it } from "vitest";
import { getMicroloopUserAgent } from "../src/utils/microloop-user-agent.ts";

describe("getMicroloopUserAgent", () => {
	it("formats the user agent expected by microloop", () => {
		const runtime = process.versions.bun ? `bun/${process.versions.bun}` : `node/${process.version}`;
		const userAgent = getMicroloopUserAgent("1.2.3");

		expect(userAgent).toBe(`microloop/1.2.3 (${process.platform}; ${runtime}; ${process.arch})`);
		expect(userAgent).toMatch(/^microloop\/[^\s()]+ \([^;()]+;\s*[^;()]+;\s*[^()]+\)$/);
	});
});
