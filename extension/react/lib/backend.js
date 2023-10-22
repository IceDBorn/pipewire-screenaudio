import { MIC_ID, readLocalStorage } from "./local-storage";

const MESSAGE_NAME = "com.icedborn.pipewirescreenaudioconnector";
const EXT_VERSION = browser.runtime.getManifest().version;

export const ERROR_VERSION_MISMATCH = "Version Mismatch";

export const EVENT_MIC_ID_UPDATED = "onMicIdUpdated";
export const EVENT_MIC_ID_REMOVED = "onMicIdRemoved";

let isStopping = false;

function enqueueCommandToBackground(command) {
  sendMessage("enqueue-command", command);
}

async function sendNativeMessage(command, args) {
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

async function sendMessage(message, command) {
  console.log("Sent message", { command, message });

  try {
    return await chrome.runtime.sendMessage({
      messageName: MESSAGE_NAME,
      message: message,
      command: command,
    });
  } catch (err) {
    console.error(
      `Failed message "${message}" with command: ${JSON.stringify(command)}`,
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

  if (message === EVENT_MIC_ID_UPDATED) {
    const micId = readLocalStorage(MIC_ID);
    const event = new CustomEvent(EVENT_MIC_ID_UPDATED, {
      detail: { micId },
    });
    document.dispatchEvent(event);
  }

  if (message === EVENT_MIC_ID_REMOVED) {
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
  return sendNativeMessage("IsPipewireScreenAudioRunning", { micId });
}

export function startPipewireScreenAudio() {
  isStopping = false;
  enqueueCommandToBackground({
    cmd: "StartPipewireScreenAudio",
    maps: { outMap: [[MIC_ID, "micId"]] }, // Set the `micId` in LocalStorage to the incoming `micId`
    event: EVENT_MIC_ID_UPDATED,
  });
}

export function stopPipewireScreenAudio(micId) {
  isStopping = true;
  enqueueCommandToBackground({
    cmd: "StopPipewireScreenAudio",
    args: { micId },
    maps: { outMap: [[MIC_ID, null]] }, // Clear the `micId` in LocalStorage
    event: EVENT_MIC_ID_REMOVED,
  });
}

export function setSharingNode(nodeSerials) {
  enqueueCommandToBackground({
    cmd: "SetSharingNode",
    args: { nodes: nodeSerials },
    maps: { inMap: [[MIC_ID, "micId"]] }, // Read the `micId` from LocalStorage and pass it as the `micId` arg
  });
}

export function shareAllDesktopAudio() {
  enqueueCommandToBackground({
    cmd: "ShareAllDesktopAudio",
    maps: { inMap: [[MIC_ID, "micId"]] }, // Read the `micId` from LocalStorage and pass it as the `micId` arg
  });
}
