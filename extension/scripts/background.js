// Any event, that is handled in here, should have a comment with the reason it is handled in here

function handleMessage(request) {
  // Asynchronously update micId in LocalStorage
  if (request.message === "sharing-started") {
    chrome.runtime
      .sendNativeMessage(request.messageName, { cmd: request.cmd })
      .then(({ micId }) => {
        window.localStorage.setItem("micId", micId);
        chrome.runtime.sendMessage("mic-id-updated");
      });
  }

  // Asynchronously update micId in LocalStorage
  if (request.message === "sharing-stopped") {
    chrome.runtime
      .sendNativeMessage(request.messageName, {
        cmd: request.cmd,
        args: request.args,
      })
      .then(() => {
        window.localStorage.setItem("micId", null);
        chrome.runtime.sendMessage("mic-id-removed");
      });
  }

  // Called from injector.js - It cannot directly call sendNativeMessage
  if (request.message === "get-session-type") {
    return chrome.runtime.sendNativeMessage(request.messageName, {
      cmd: "GetSessionType",
      args: [],
    });
  }
}

chrome.runtime.onMessage.addListener(handleMessage);
