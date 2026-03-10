// Any event, that is handled in here, should have a comment with the reason it is handled in here

let commandsQueue = [];
let commandsQueueRunning = false;

async function sendNativeMessage(messageName, cmd, args) {
	const response = await chrome.runtime.sendNativeMessage(messageName, { cmd, args });
	if (!response.success) {
		throw new Error(`unsuccessful message response during call to ${cmd} with arguments ${JSON.stringify(args)}: ${response.errorMessage}`);
	}
	return response.response;
}

async function runQueuedCommands() {
	commandsQueueRunning = true;

	while (commandsQueue.length) {
		try {
			const command = commandsQueue.shift();
			const args = { ...command.args };
			const { inMap, outMap } = command.maps || {};

			if (inMap) {
				inMap.forEach(([storageKey, argKey]) => {
					args[argKey] = window.localStorage.getItem(storageKey);
				});
			}

			console.log(args);
			const result = await sendNativeMessage(
				command.messageName,
				command.cmd,
				args,
				command.maps,
			);
			console.log(result);

			if (outMap) {
				outMap.forEach(([storageKey, resultKey]) => {
					window.localStorage.setItem(
						storageKey,
						resultKey ? result[resultKey] : null,
					);
				});
			}

			if (command.event) {
				chrome.runtime.sendMessage(command.event);
			}
		} catch (err) {
			console.error(err);
		}
	}

	commandsQueueRunning = false;
}

function handleMessage(request) {
	// Run multiple commands sequentially
	if (request.message === "enqueue-command") {
		commandsQueue.push({
			...request.command,
			messageName: request.messageName,
		});
		if (!commandsQueueRunning) {
			runQueuedCommands();
		}
	}

	// Called from injector.js - It cannot directly call sendNativeMessage
	if (request.message === "get-session-type") {
		return sendNativeMessage(request.messageName, "GetSessionType");
	}

	if (request.message === "instance-identifier") {
		return sendNativeMessage(request.messageName, "SetInstanceIdentifier", {
			id: request.instanceIdentifier,
		});
	}
}

chrome.runtime.onMessage.addListener(handleMessage);
