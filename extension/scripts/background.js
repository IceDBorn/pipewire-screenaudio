function handleMessage(response) {
  if (response.message === 'node-shared') {
    // Passthrough the selected node to pipewire-screenaudio
    chrome.runtime.sendNativeMessage(response.messageName, { cmd: response.cmd, args: response.args })
      .then(({ micPid }) => {
        window.localStorage.setItem('micPid', micPid)
        chrome.runtime.sendMessage('pid-updated')
      })
  }
}

chrome.runtime.onMessage.addListener(handleMessage)
