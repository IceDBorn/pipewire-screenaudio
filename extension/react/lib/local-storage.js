export const SELECTED_ROWS = "selectedRows";
export const MIC_ID = "micId";
export const ALL_DESKTOP = "allDesktopAudio";

export function readLocalStorage(name) {
  try {
    return JSON.parse(window.localStorage.getItem(name));
  } catch {
    return null;
  }
}

export function updateLocalStorage(name, value) {
  window.localStorage.setItem(name, JSON.stringify(value));
}
