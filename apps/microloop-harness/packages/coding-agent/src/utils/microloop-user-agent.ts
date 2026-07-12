export function getMicroloopUserAgent(version: string): string {
	const runtime = process.versions.bun ? `bun/${process.versions.bun}` : `node/${process.version}`;
	return `microloop/${version} (${process.platform}; ${runtime}; ${process.arch})`;
}
