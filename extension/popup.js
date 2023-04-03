const MESSAGE_NAME = 'com.icedborn.screenaudiomicconnector'

let selectedNode = null

async function isRunning () {
  const micPid = window.localStorage.getItem('micPid')
  if (!micPid) {
    return false
  }

  const { isRunning } = await chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'IsVirtmicRunning', args: [{ micPid }] })

  if (!isRunning) {
    window.localStorage.setItem('micPid', null)
  }

  return isRunning
}

function createShareBtn (root) {
  const shareBtn = document.createElement('button')
  shareBtn.style.background = '#202324'
  shareBtn.style.color = '#e8e6e3'
  shareBtn.id = 'share-btn'
  shareBtn.innerText = 'Share'
  root.appendChild(shareBtn)

  const shareBtnEl = document.getElementById('share-btn')
  shareBtnEl.addEventListener('click', () => {
    window.localStorage.setItem('selectedNode', selectedNode)
    chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'StartVirtmic', args: [{ node: selectedNode }] })
      .then(({ micPid }) => {
        root.removeChild(shareBtnEl)
        window.localStorage.setItem('micPid', micPid)
        updateGui(root)
      })
  })
}

function createStopBtn (root) {
  const stopBtn = document.createElement('button')
  stopBtn.style.background = '#202324'
  stopBtn.style.color = '#e8e6e3'
  stopBtn.id = 'stop-btn'
  stopBtn.innerText = 'Stop'
  root.appendChild(stopBtn)

  const stopBtnEl = document.getElementById('stop-btn')
  stopBtnEl.addEventListener('click', async () => {
    if (await isRunning()) {
      const micPid = window.localStorage.getItem('micPid')
      chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'StopVirtmic', args: [{ micPid }] })
        .then(() => {
          root.removeChild(stopBtnEl)
          window.localStorage.setItem('micPid', null)
          updateGui(root)
        })
    }
  })
}

async function updateGui (root) {
  if (await isRunning()) {
    document.getElementById('is-running').innerText = `screenaudio-mic is running with PID: ${window.localStorage.getItem('micPid')}`
    createStopBtn(root)
  } else {
    document.getElementById('is-running').innerText = 'screenaudio-mic is not running'
    createShareBtn(root)
  }
}

function onResponse (response) {
  const ALL_DESKTOP_AUDIO_TEXT = 'All Desktop Audio'

  const dropdown = document.getElementById('dropdown')
  const allDesktopAudioOption = document.createElement('option')

  allDesktopAudioOption.innerText = ALL_DESKTOP_AUDIO_TEXT
  allDesktopAudioOption.value = ALL_DESKTOP_AUDIO_TEXT
  dropdown.appendChild(allDesktopAudioOption)

  for (const element of response) {
    const option = document.createElement('option')
    option.innerText = element
    option.value = element
    dropdown.appendChild(option)
  }

  const lastSelection = window.localStorage.getItem('selectedNode')
  if (lastSelection) {
    dropdown.value = lastSelection
  }

  selectedNode = dropdown.value
  dropdown.addEventListener('change', () => { selectedNode = dropdown.value })

  const root = document.getElementById('root')
  updateGui(root)

  document.getElementById('heading').innerText = 'Select audio node to share'
}

function onError (error) {
  console.error(error)
  document.getElementById('heading').innerText = 'The native connector is missing!'
  document.getElementById('is-running').hidden = true
  document.getElementById('dropdown').hidden = true
}

// let sending = chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: "StartVirtmic", args: [{ node: '' }] });
// let sending = chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: "StopVirtmic", args: [{ micPid: 0 }] });
const sending = chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'GetNodes', args: [] })
sending.then(onResponse, onError)
