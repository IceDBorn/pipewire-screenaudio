const MESSAGE_NAME = 'com.icedborn.screenaudiomicconnector'

let selectedNode = null

function createShareBtn (root) {
  const shareBtn = document.createElement('button')
  shareBtn.style.background = '#202324'
  shareBtn.style.color = '#e8e6e3'
  shareBtn.id = 'share-btn'
  shareBtn.innerText = 'Share'
  root.appendChild(shareBtn)

  const shareBtnEl = document.getElementById('share-btn')
  shareBtnEl.addEventListener('click', () => {
    chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'StartVirtmic', args: [{ node: selectedNode }] })
      .then(({ micPid }) => {
        root.removeChild(shareBtnEl)
        createStopBtn(root, micPid)
      })
  })
}

function createStopBtn (root, micPid) {
  const stopBtn = document.createElement('button')
  stopBtn.style.background = '#202324'
  stopBtn.style.color = '#e8e6e3'
  stopBtn.id = 'stop-btn'
  stopBtn.innerText = 'Stop'
  root.appendChild(stopBtn)

  const stopBtnEl = document.getElementById('stop-btn')
  stopBtnEl.addEventListener('click', () => {
    chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'StopVirtmic', args: [{ micPid }] })
      .then(({ micPid, micId }) => {
        root.removeChild(stopBtnEl)
        createShareBtn(root)
      })
  })
}

function onResponse (response) {
  const ALL_DESKTOP_AUDIO_TEXT = 'All Desktop Audio'

  const dropdown = document.getElementById('dropdown')
  const allDesktopAudioOption = document.createElement('option')

  allDesktopAudioOption.innerText = ALL_DESKTOP_AUDIO_TEXT
  allDesktopAudioOption.value = ALL_DESKTOP_AUDIO_TEXT
  dropdown.appendChild(allDesktopAudioOption)
  dropdown.addEventListener('change', () => { selectedNode = dropdown.value })

  const root = document.getElementById('root')
  createShareBtn(root)

  for (const element of response) {
    const option = document.createElement('option')
    option.innerText = element
    option.value = element
    dropdown.appendChild(option)
  }

  document.getElementById('heading').innerText = 'Select audio node to share'
}

function onError (error) {
  console.error(error)
  document.getElementById('heading').innerText = 'The native connector is missing!'
  document.getElementById('dropdown').hidden = true
}

// let sending = chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: "StartVirtmic", args: [{ node: '' }] });
// let sending = chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: "StopVirtmic", args: [{ micPid: 0 }] });
const sending = chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'GetNodes', args: [] })
sending.then(onResponse, onError)
