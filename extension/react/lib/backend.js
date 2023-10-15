const MESSAGE_NAME = "com.icedborn.pipewirescreenaudioconnector";
const EXT_VERSION = browser.runtime.getManifest().version;

export const ERROR_VERSION_MISMATCH = "Version Mismatch";

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
  sendMessage("StartPipewireScreenAudio", "sharing-started");
}

export async function stopPipewireScreenAudio(micId) {
  isStopping = true;
  sendMessage("StopPipewireScreenAudio", "sharing-stopped", [{ micId }]);
}

export async function setSharingNode(micId, nodeSerials) {
  return sendNativeMessage("SetSharingNode", [{ micId, nodes: nodeSerials }]);
}

// Remove nodes item after all desktop audio rework
export async function shareAllDesktopAudio(micId) {
  return sendNativeMessage("ShareAllDesktopAudio", [{ micId, nodes: [-1] }]);
}
