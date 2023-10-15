import { MIC_ID, readLocalStorage } from "./local-storage";

const MESSAGE_NAME = "com.icedborn.pipewirescreenaudioconnector";
const EXT_VERSION = browser.runtime.getManifest().version;

export const ERROR_VERSION_MISMATCH = "Version Mismatch";

export const EVENT_MIC_ID_UPDATED = "onMicIdUpdated";
export const EVENT_MIC_ID_REMOVED = "onMicIdRemoved";

let isStopping = false;

async function sendNativeMessage(command, args = []) {
  console.log("Sent native message", { command, args });

  try {
    return await chrome.runtime.sendNativeMessage(MESSAGE_NAME, {
      cmd: command,
      args: args,
    });
  } catch (err) {
    console.error(
      `Failed native "${command}" with args: ${JSON.stringify(args)}`,
    );
    throw err;
  }
}

async function sendMessage(command, message, args) {
  console.log("Sent message", { command, message, args });

  try {
    return await chrome.runtime.sendMessage({
      messageName: MESSAGE_NAME,
      message: message,
      cmd: command,
      args: args,
    });
  } catch (err) {
    console.error(
      `Failed message "${message}" with args: ${JSON.stringify(args)}`,
    );
    throw err;
  }
}

function matchVersion(a, b) {
  const aSplit = a.split(".");
  const bSplit = b.split(".");
  return aSplit[0] === bSplit[0] && aSplit[1] === bSplit[1];
}

function getNewPromise() {
  let resolvePromise, rejectPromise;
  const promise = new Promise((resolve, reject) => {
    resolvePromise = resolve;
    rejectPromise = reject;
  });

  return { promise, resolvePromise, rejectPromise };
}

function handleMessage(message) {
  console.log({ message });

  if (message === "mic-id-updated") {
    const micId = readLocalStorage(MIC_ID);
    const event = new CustomEvent(EVENT_MIC_ID_UPDATED, {
      detail: { micId },
    });
    document.dispatchEvent(event);
  }

  if (message === "mic-id-removed") {
    const event = new CustomEvent(EVENT_MIC_ID_REMOVED);
    document.dispatchEvent(event);
  }
}

chrome.runtime.onMessage.addListener(handleMessage);

export async function healthCheck() {
  const { version: nativeVersion } = await sendNativeMessage("GetVersion");

  if (!matchVersion(nativeVersion, EXT_VERSION)) {
    throw new Error(ERROR_VERSION_MISMATCH, {
      cause: {
        nativeVersion,
        extensionVersion: EXT_VERSION,
      },
    });
  }

  return true;
}

export async function getNodes() {
  return sendNativeMessage("GetNodes");
}

export async function isPipewireScreenAudioRunning(micId) {
  return sendNativeMessage("IsPipewireScreenAudioRunning", [{ micId }]);
}

export async function startPipewireScreenAudio() {
  isStopping = false;

  const { promise, resolvePromise, rejectPromise } = getNewPromise();
  const listener = (event) => resolvePromise(event.detail.micId);
  document.addEventListener(EVENT_MIC_ID_UPDATED, listener, { once: true });

  try {
    await sendMessage("StartPipewireScreenAudio", "sharing-started");
  } catch (err) {
    rejectPromise(err);
  }

  return promise;
}

export async function stopPipewireScreenAudio(micId) {
  isStopping = true;

  const { promise, resolvePromise, rejectPromise } = getNewPromise();
  const listener = (micId) => resolvePromise(micId);
  document.addEventListener(EVENT_MIC_ID_UPDATED, listener, { once: true });

  try {
    await sendMessage("StopPipewireScreenAudio", "sharing-stopped", [
      { micId },
    ]);
  } catch (err) {
    rejectPromise(err);
  }

  return promise;
}

export async function setSharingNode(micId, nodeSerials) {
  return sendNativeMessage("SetSharingNode", [{ micId, nodes: nodeSerials }]);
}

// Remove nodes item after all desktop audio rework
export async function shareAllDesktopAudio(micId) {
  return sendNativeMessage("ShareAllDesktopAudio", [{ micId, nodes: [-1] }]);
}
