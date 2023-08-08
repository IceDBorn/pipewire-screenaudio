const MESSAGE_NAME = 'com.icedborn.pipewirescreenaudioconnector'

const nullthrows = (v) => {
  if (v == null) throw new Error('null')
  return v
}

function injectCode (src) {
  const script = document.createElement('script')
  script.src = src
  script.onload = function () {
    console.log('pipewire-screenaudio script injected')

    browser.runtime
      .sendMessage({ messageName: MESSAGE_NAME, message: 'get-session-type' })
      .then(({ type }) => {
        console.debug(type)
        window.postMessage({ message: "set-session-type", type })
      });

    this.remove()
  }

  nullthrows(document.head || document.documentElement).appendChild(script)
}

injectCode(chrome.runtime.getURL('/scripts/index.js'))
