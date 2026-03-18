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
		/**
		 * @type {typeof chrome.storage.local.get<Partial<LocalStorageTypes>>}
		 */
		const chromeStorageLocalGet = chrome.storage.local.get;
		const stored = await chromeStorageLocalGet(name);
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
