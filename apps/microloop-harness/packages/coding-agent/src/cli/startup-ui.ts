import { ProcessTerminal, setKeybindings, TUI } from "@microloop/tui";
import { existsSync } from "fs";
import { APP_NAME, CONFIG_DIR_NAME, ENV_AGENT_DIR, getAgentDir, getSettingsPath, PACKAGE_NAME } from "../config.ts";

import { AuthStorage } from "../core/auth-storage.ts";
import { KeybindingsManager } from "../core/keybindings.ts";
import { DefaultPackageManager, type ResolvedResource } from "../core/package-manager.ts";
import { SettingsManager } from "../core/settings-manager.ts";
import { ExtensionInputComponent } from "../modes/interactive/components/extension-input.ts";
import { ExtensionSelectorComponent } from "../modes/interactive/components/extension-selector.ts";
import {
	FirstTimeSetupComponent,
	type FirstTimeSetupResult,
} from "../modes/interactive/components/first-time-setup.ts";
import {
	detectTerminalBackgroundFromEnv,
	detectTerminalThemeForAuto,
	initTheme,
	loadThemeFromPath,
	parseAutoThemeSetting,
	resolveThemeSetting,
	setRegisteredThemes,
	setTheme,
	type Theme,
} from "../modes/interactive/theme/theme.ts";

// Common LLM providers that use API key auth
const API_KEY_PROVIDERS = [
	{ id: "openai", name: "OpenAI", keyHint: "sk-..." },
	{ id: "anthropic", name: "Anthropic", keyHint: "sk-ant-..." },
	{ id: "google", name: "Google Gemini", keyHint: "AIza..." },
	{ id: "groq", name: "Groq", keyHint: "gsk_..." },
	{ id: "deepseek", name: "DeepSeek", keyHint: "sk-..." },
	{ id: "mistral", name: "Mistral", keyHint: "..." },
	{ id: "openrouter", name: "OpenRouter", keyHint: "sk-or-..." },
	{ id: "fireworks", name: "Fireworks", keyHint: "fw_..." },
	{ id: "together", name: "Together AI", keyHint: "..." },
	{ id: "xai", name: "xAI Grok", keyHint: "..." },
	{ id: "cerebras", name: "Cerebras", keyHint: "..." },
];

/**
 * Show an interactive provider setup dialog after first-time setup.
 * Lets the user pick a provider and enter an API key, then saves to auth.json.
 */
export async function showProviderSetup(settingsManager: SettingsManager): Promise<void> {
	const selectedProvider = await showStartupSelector(
		settingsManager,
		`${APP_NAME} - Configure a provider\nSelect a provider to set up your API key. You can add more later with /login.`,
		API_KEY_PROVIDERS.map((p) => ({ label: p.name, value: p.id })),
	);

	if (!selectedProvider) return;

	const provider = API_KEY_PROVIDERS.find((p) => p.id === selectedProvider);
	if (!provider) return;

	const apiKey = await showStartupInput(
		settingsManager,
		`Enter your ${provider.name} API key\n(e.g., ${provider.keyHint})`,
		provider.keyHint,
	);

	if (!apiKey || apiKey.trim() === "") return;

	// Save the API key to auth.json
	try {
		const authStorage = AuthStorage.create();
		authStorage.set(provider.id, { type: "api_key", key: apiKey.trim() });
		console.error(`[${APP_NAME}] API key saved for ${provider.name}.`);
	} catch (error) {
		console.error(`[${APP_NAME}] Failed to save API key: ${error}`);
	}
}

// Microloop distribution identifiers
const MICROLOOP_PACKAGE_NAME = "@microloop/coding-agent";
const MICROLOOP_APP_NAME = "microloop";
const MICROLOOP_CONFIG_DIR_NAME = ".microloop";

interface DistributionMetadata {
	packageName: string;
	appName: string;
	configDirName: string;
}

function isRecognizedDistribution({ packageName, appName, configDirName }: DistributionMetadata): boolean {
	// Microloop distribution
	if (packageName === MICROLOOP_PACKAGE_NAME && appName === MICROLOOP_APP_NAME && configDirName === MICROLOOP_CONFIG_DIR_NAME) {
		return true;
	}
	return false;
}

function loadThemes(resources: ResolvedResource[]): Theme[] {
	const themes: Theme[] = [];
	const seen = new Set<string>();
	for (const resource of resources) {
		if (!resource.enabled) continue;
		try {
			const loadedTheme = loadThemeFromPath(resource.path);
			if (loadedTheme.name) {
				if (seen.has(loadedTheme.name)) continue;
				seen.add(loadedTheme.name);
			}
			themes.push(loadedTheme);
		} catch {
			// Startup prompts should not fail because a theme is broken. The normal
			// resource loader reports theme diagnostics later in startup.
		}
	}
	return themes;
}

async function loadStartupThemes(settingsManager: SettingsManager): Promise<Theme[]> {
	const globalSettingsManager = SettingsManager.inMemory(settingsManager.getGlobalSettings(), {
		projectTrusted: false,
	});
	const packageManager = new DefaultPackageManager({
		cwd: process.cwd(),
		agentDir: getAgentDir(),
		settingsManager: globalSettingsManager,
	});
	const resolvedPaths = await packageManager.resolve(async () => "skip");
	return loadThemes(resolvedPaths.themes);
}

export async function createStartupTui(settingsManager: SettingsManager): Promise<TUI> {
	setRegisteredThemes(await loadStartupThemes(settingsManager));
	const terminalTheme = detectTerminalBackgroundFromEnv().theme;
	initTheme(resolveThemeSetting(settingsManager.getThemeSetting(), terminalTheme) ?? terminalTheme);
	setKeybindings(KeybindingsManager.create());
	const ui = new TUI(new ProcessTerminal(), settingsManager.getShowHardwareCursor());
	ui.setClearOnShrink(settingsManager.getClearOnShrink());
	return ui;
}

export function startStartupTui(ui: TUI, settingsManager: SettingsManager): void {
	ui.start();
	void applyDetectedStartupTheme(ui, settingsManager);
}

async function applyDetectedStartupTheme(ui: TUI, settingsManager: SettingsManager): Promise<void> {
	const themeSetting = settingsManager.getThemeSetting();
	if (themeSetting && !parseAutoThemeSetting(themeSetting)) return;

	const terminalTheme = await detectTerminalThemeForAuto({ ui, timeoutMs: 100 });
	setTheme(resolveThemeSetting(themeSetting, terminalTheme) ?? terminalTheme);
	ui.invalidate();
	ui.requestRender();
}

async function clearStartupTui(ui: TUI): Promise<void> {
	ui.clear();
	ui.requestRender();
	await new Promise((resolve) => setTimeout(resolve, 25));
}

/**
 * First-time setup runs when all of these hold:
 * - this is a recognized distribution (Microloop)
 * - the default agent directory is used (no custom agent dir override)
 * - setup was not completed before (settings.json does not exist)
 */
export function shouldRunFirstTimeSetup(settingsPath: string = getSettingsPath()): boolean {
	if (
		!isRecognizedDistribution({
			packageName: PACKAGE_NAME,
			appName: APP_NAME,
			configDirName: CONFIG_DIR_NAME,
		})
	) {
		return false;
	}
	if (process.env[ENV_AGENT_DIR]) {
		return false;
	}
	return !existsSync(settingsPath);
}

export async function showStartupSelector<T>(
	settingsManager: SettingsManager,
	title: string,
	options: Array<{ label: string; value: T }>,
): Promise<T | undefined> {
	const ui = await createStartupTui(settingsManager);
	return new Promise((resolve) => {
		let settled = false;
		const finish = async (result: T | undefined) => {
			if (settled) {
				return;
			}
			settled = true;
			await clearStartupTui(ui);
			ui.stop();
			resolve(result);
		};

		const selector = new ExtensionSelectorComponent(
			title,
			options.map((option) => option.label),
			(option) => void finish(options.find((entry) => entry.label === option)?.value),
			() => void finish(undefined),
			{ tui: ui },
		);
		ui.addChild(selector);
		ui.setFocus(selector);
		startStartupTui(ui, settingsManager);
	});
}

/** Show the first-time setup dialog and persist the result */
export async function showFirstTimeSetup(settingsManager: SettingsManager): Promise<void> {
	const ui = await createStartupTui(settingsManager);
	return new Promise((resolve) => {
		let settled = false;
		const finish = async (result: FirstTimeSetupResult | undefined) => {
			if (settled) {
				return;
			}
			settled = true;
			if (result) {
				settingsManager.setTheme(result.theme);
				settingsManager.setEnableAnalytics(result.shareAnalytics);
				await settingsManager.flush();
			}
			await clearStartupTui(ui);
			ui.stop();
			resolve();
		};

		const showSetup = async () => {
			ui.start();
			const detectedTheme = await detectTerminalThemeForAuto({ ui, timeoutMs: 100 });
			setTheme(detectedTheme);
			const component = new FirstTimeSetupComponent({
				detectedTheme,
				onThemePreview: (themeName) => {
					setTheme(themeName);
					ui.requestRender();
				},
				onSubmit: (result) => void finish(result),
				onCancel: () => void finish(undefined),
			});
			ui.addChild(component);
			ui.setFocus(component);
			ui.requestRender();
		};

		void showSetup();
	});
}

export async function showStartupInput(
	settingsManager: SettingsManager,
	title: string,
	placeholder?: string,
): Promise<string | undefined> {
	const ui = await createStartupTui(settingsManager);
	return new Promise((resolve) => {
		let settled = false;
		const finish = async (result: string | undefined) => {
			if (settled) {
				return;
			}
			settled = true;
			input.dispose();
			await clearStartupTui(ui);
			ui.stop();
			resolve(result);
		};

		const input = new ExtensionInputComponent(
			title,
			placeholder,
			(value) => void finish(value),
			() => void finish(undefined),
			{
				tui: ui,
			},
		);
		ui.addChild(input);
		ui.setFocus(input);
		startStartupTui(ui, settingsManager);
	});
}
