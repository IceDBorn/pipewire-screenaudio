export const SELECTED_NODES = "selectedNodes";
export const MIC_ID = "micId";
export const ALL_DESKTOP = "allDesktopAudio";

/** @import { LocalStorageTypes } from "./types" */

/**
 * @template {keyof LocalStorageTypes} T
 * @param {T} name
 * @returns {Promise<LocalStorageTypes[T] | null>}
 */
export async function readLocalStorage(name) {
	try {
		const stored = await chrome.storage.local.get(name);
		return stored[name] ?? null;
	} catch {
		return null;
	}
}

/**
 * @template {keyof LocalStorageTypes} T
 * @param {T} name
 * @param {LocalStorageTypes[T] | null} value
 */
export async function updateLocalStorage(name, value) {
	await chrome.storage.local.set({ [name]: value });
}
