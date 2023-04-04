const MESSAGE_NAME = 'com.icedborn.pipewirescreenaudioconnector'
const ALL_DESKTOP_AUDIO_TEXT = 'All Desktop Audio'

const dropdown = document.getElementById('dropdown')
const heading = document.getElementById('heading')

let selectedNode = null

async function isRunning () {
  const micPid = window.localStorage.getItem('micPid')
  if (!micPid) {
    return false
  }

  const { isRunning } = await chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'IsPipewireScreenAudioRunning', args: [{ micPid }] })

  if (!isRunning) {
    window.localStorage.setItem('micPid', null)
  }

  return isRunning
}

function createShareBtn (root) {
  const shareBtn = document.createElement('button')
  shareBtn.id = 'share-btn'
  shareBtn.className = 'button'
  shareBtn.innerText = 'Share'
  root.appendChild(shareBtn)

  const shareBtnEl = document.getElementById('share-btn')
  shareBtnEl.addEventListener('click', () => {
    window.localStorage.setItem('selectedNode', selectedNode)
    chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'StartPipewireScreenAudio', args: [{ node: selectedNode }] })
      .then(({ micPid }) => {
        root.removeChild(shareBtnEl)
        window.localStorage.setItem('micPid', micPid)
        updateGui(root)
      })
  })
}

function createStopBtn (root) {
  const stopBtn = document.createElement('button')
  stopBtn.id = 'stop-btn'
  stopBtn.className = 'button'
  stopBtn.innerText = 'Stop'
  root.appendChild(stopBtn)

  const stopBtnEl = document.getElementById('stop-btn')
  stopBtnEl.addEventListener('click', async () => {
    if (await isRunning()) {
      const micPid = window.localStorage.getItem('micPid')
      chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'StopPipewireScreenAudio', args: [{ micPid }] })
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
    heading.innerText = `Pipewire Screenaudio is running with PID: ${window.localStorage.getItem('micPid')}`
    dropdown.hidden = true
    createStopBtn(root)
  } else {
    heading.innerText = `Select audio node to share`
    dropdown.hidden = false
    createShareBtn(root)
  }
}

function onResponse (response) {
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

  heading.innerText = 'Select audio node to share'
}

function onError (error) {
  console.error(error)
  heading.innerText = 'The native connector is missing!'
  dropdown.hidden = true
}

const sending = chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'GetNodes', args: [] })
sending.then(onResponse, onError)
