import { MIC_ID, readLocalStorage } from "./local-storage";

const MESSAGE_NAME = "com.icedborn.pipewirescreenaudioconnector";
const EXT_VERSION = chrome.runtime.getManifest().version;

export const ERROR_VERSION_MISMATCH = "Version Mismatch";

export const EVENT_MIC_ID_UPDATED = "onMicIdUpdated";
export const EVENT_MIC_ID_REMOVED = "onMicIdRemoved";

let isStopping = false;

import * as NativeMessaging from "./nativeMessageTypes";
import * as BackendTypes from "./backendTypes";
import { MicIdUpdatedEvent } from "./types";

function enqueueCommandToBackground<Command extends NativeMessaging.Commands>(
	command: BackendTypes.BackgroundCommand<Command>,
) {
	sendMessage("enqueue-command", command);
}

async function sendNativeMessage<Command extends NativeMessaging.Commands>(
	command: Command,
	args: NativeMessaging.Requests[Command],
): Promise<NativeMessaging.Responses[Command]> {
	console.log("Sent native message", { command, args });

	const response = await chrome.runtime.sendNativeMessage(MESSAGE_NAME, {
		cmd: command,
		args: args,
	});
	// support previous connector versions
	if (response.success === undefined) {
		return response;
	}
	if (!response.success) {
		throw new Error(
			`unsuccessful message response during call to ${command} with arguments ${JSON.stringify(args)}: ${response.errorMessage}`,
		);
	}
	return response.response;
}

async function sendMessage(message: string, command: any) {
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

function matchVersion(a: string, b: string) {
	const aSplit = a.split(".");
	const bSplit = b.split(".");
	return aSplit[0] === bSplit[0] && aSplit[1] === bSplit[1];
}

function handleMessage(message: string) {
	console.log({ message });

	if (message === EVENT_MIC_ID_UPDATED) {
		readLocalStorage(MIC_ID).then((micId: number) => {
			if (micId === null) return;
			const event: MicIdUpdatedEvent = new CustomEvent(EVENT_MIC_ID_UPDATED, {
				detail: { micId },
			});
			document.dispatchEvent(event);
		});
	}

	if (message === EVENT_MIC_ID_REMOVED) {
		const event = new CustomEvent(EVENT_MIC_ID_REMOVED);
		document.dispatchEvent(event);
	}
}

chrome.runtime.onMessage.addListener(handleMessage);

export async function healthCheck() {
	const { version: nativeVersion } = await sendNativeMessage(
		"GetVersion",
		undefined,
	);

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
	return await sendNativeMessage("GetNodes", undefined);
}

export async function isPipewireScreenAudioRunning(micId: number) {
	return (await sendNativeMessage("IsPipewireScreenAudioRunning", { micId }))
		.isRunning;
}

export function startPipewireScreenAudio() {
	isStopping = false;
	enqueueCommandToBackground({
		cmd: "StartPipewireScreenAudio",
		args: undefined,
		maps: { outMap: [[MIC_ID, "micId"]] }, // Set the `micId` in LocalStorage to the incoming `micId`
		event: EVENT_MIC_ID_UPDATED,
	});
}

export function stopPipewireScreenAudio(micId: number) {
	isStopping = true;
	enqueueCommandToBackground({
		cmd: "StopPipewireScreenAudio",
		args: { micId },
		maps: { outMap: [[MIC_ID, null]] }, // Clear the `micId` in LocalStorage
		event: EVENT_MIC_ID_REMOVED,
	});
}

export function setSharingNode(nodeSerials: number[]) {
	enqueueCommandToBackground({
		cmd: "SetSharingNode",
		args: { nodes: nodeSerials },
		maps: { inMap: [[MIC_ID, "micId"]] }, // Read the `micId` from LocalStorage and pass it as the `micId` arg
	});
}

export const isChromium = () => typeof browser === "undefined";

export const isIncognito = () => {
	if (isChromium()) {
		return false;
	}

	return !!browser?.extension.inIncognitoContext;
};
