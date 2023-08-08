function handleMessage(response) {
  if (response.message === 'sharing-started') {
    // Start pipewire-screenaudio
    chrome.runtime.sendNativeMessage(response.messageName, { cmd: response.cmd })
      .then(({ micId }) => {
        window.localStorage.setItem('micId', micId)
        chrome.runtime.sendMessage('mic-id-updated')
        // Passthrough the selected node to pipewire-screenaudio
        chrome.runtime.sendNativeMessage(response.messageName, { cmd: 'SetSharingNode', args: [{ node: response.args[0].node, micId }] })
      })
  }

  if (response.message === 'node-changed') {
    // Passthrough the selected node to pipewire-screenaudio
    chrome.runtime.sendNativeMessage(response.messageName, { cmd: response.cmd, args: response.args })
  }

  if (response.message === 'sharing-stopped') {
    chrome.runtime.sendNativeMessage(response.messageName, { cmd: response.cmd, args: response.args })
      .then(() => {
        window.localStorage.setItem('micId', null)
        chrome.runtime.sendMessage('mic-id-removed')
      })
  }

  if (response.message === 'get-session-type') {
    return chrome.runtime.sendNativeMessage(response.messageName, { cmd: 'GetSessionType', args: [] })
  }
}

chrome.runtime.onMessage.addListener(handleMessage)
