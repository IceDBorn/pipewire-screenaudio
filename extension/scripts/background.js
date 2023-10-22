// Any event, that is handled in here, should have a comment with the reason it is handled in here

let commandsQueue = [];
let commandsQueueRunning = false;

function sendNativeMessage(messageName, cmd, args) {
  return chrome.runtime.sendNativeMessage(messageName, {
    cmd,
    args: args ? [args] : undefined,
  });
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

      console.log(args)
      const result = await sendNativeMessage(command.messageName, command.cmd, args, command.maps)
      console.log(result)

      if (outMap) {
        outMap.forEach(([storageKey, resultKey]) => {
          window.localStorage.setItem(storageKey, (resultKey ? result[resultKey] : null));
        });
      }

      if (command.event) {
        chrome.runtime.sendMessage(command.event);
      }
    } catch (err) {
      console.error(err)
    }
  }

  commandsQueueRunning = false;
}

function handleMessage(request) {
  switch (request.message) {
    // Run multiple commands sequentially
    case "enqueue-command":
      commandsQueue.push({
        ...request.command,
        messageName: request.messageName,
      });
      if (!commandsQueueRunning) {
        runQueuedCommands();
      }
      break;

    // Called from injector.js - It cannot directly call sendNativeMessage
    case "get-session-type":
      sendNativeMessage(request.messageName, "GetSessionType");
      break;

    default:
      throw `Unknown message "${request.message}"`;
  }
}

chrome.runtime.onMessage.addListener(handleMessage);
