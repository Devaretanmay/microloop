export function areExperimentalFeaturesEnabled(): boolean {
	return process.env.MICROLOOP_EXPERIMENTAL === "1";
}
