export const SELECTED_NODES = "selectedNodes";
export const MIC_ID = "micId";
export const ALL_DESKTOP = "allDesktopAudio";
import type { LocalStorageTypes } from "./types";

export async function readLocalStorage<T extends keyof LocalStorageTypes>(
	name: T,
): Promise<LocalStorageTypes[T] | null> {
	try {
		const stored =
			await chrome.storage.local.get<Partial<LocalStorageTypes>>(name);
		return stored[name] ?? null;
	} catch {
		return null;
	}
}

export async function updateLocalStorage<T extends keyof LocalStorageTypes>(
	name: T,
	value: LocalStorageTypes[T] | null,
) {
	await chrome.storage.local.set({ [name]: value });
}
