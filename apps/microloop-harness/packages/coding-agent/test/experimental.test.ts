import { afterEach, describe, expect, it } from "vitest";
import { areExperimentalFeaturesEnabled } from "../src/core/experimental.ts";

describe("areExperimentalFeaturesEnabled", () => {
	const originalPiExperimental = process.env.MICROLOOP_EXPERIMENTAL;

	afterEach(() => {
		if (originalPiExperimental === undefined) {
			delete process.env.MICROLOOP_EXPERIMENTAL;
		} else {
			process.env.MICROLOOP_EXPERIMENTAL = originalPiExperimental;
		}
	});

	it("returns false when MICROLOOP_EXPERIMENTAL is unset", () => {
		delete process.env.MICROLOOP_EXPERIMENTAL;

		expect(areExperimentalFeaturesEnabled()).toBe(false);
	});

	it("returns false when MICROLOOP_EXPERIMENTAL is empty", () => {
		process.env.MICROLOOP_EXPERIMENTAL = "";

		expect(areExperimentalFeaturesEnabled()).toBe(false);
	});

	it("returns true when MICROLOOP_EXPERIMENTAL is set to 1", () => {
		process.env.MICROLOOP_EXPERIMENTAL = "1";

		expect(areExperimentalFeaturesEnabled()).toBe(true);
	});

	it("returns false when MICROLOOP_EXPERIMENTAL is set to 0", () => {
		process.env.MICROLOOP_EXPERIMENTAL = "0";

		expect(areExperimentalFeaturesEnabled()).toBe(false);
	});

	it("returns false when MICROLOOP_EXPERIMENTAL is set to a non-1 value", () => {
		process.env.MICROLOOP_EXPERIMENTAL = "true";

		expect(areExperimentalFeaturesEnabled()).toBe(false);
	});
});
