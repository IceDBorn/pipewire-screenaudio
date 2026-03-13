export const SELECTED_NODES = "selectedNodes";
export const MIC_ID = "micId";
export const ALL_DESKTOP = "allDesktopAudio";

export async function readLocalStorage(name) {
	try {
		const stored = await chrome.storage.local.get(name);
		return stored[name] ?? null;
	} catch {
		return null;
	}
}

export async function updateLocalStorage(name, value) {
	await chrome.storage.local.set({ [name]: value });
}
