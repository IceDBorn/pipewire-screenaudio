function handleMessage (response) {
  if (response.message === 'node-shared') {
    // Passthrough the selected node to pipewire-screenaudio
    chrome.runtime.sendNativeMessage(response.messageName, { cmd: response.cmd, args: response.args })
      .then(({ micId }) => {
        window.localStorage.setItem('micId', micId)
        chrome.runtime.sendMessage('mic-id-updated')
      })
  }

  if (response.message === 'node-stopped') {
    chrome.runtime.sendNativeMessage(response.messageName, { cmd: response.cmd, args: response.args })
      .then(() => {
        window.localStorage.setItem('micId', null)
        chrome.runtime.sendMessage('mic-id-removed')
      })
  }
}

chrome.runtime.onMessage.addListener(handleMessage)
