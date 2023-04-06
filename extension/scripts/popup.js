const MESSAGE_NAME = 'com.icedborn.pipewirescreenaudioconnector'
const ALL_DESKTOP_AUDIO_TEXT = 'All Desktop Audio'

const dropdown = document.getElementById('dropdown')
const message = document.getElementById('message')

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
  shareBtn.className = 'btn btn-success'
  shareBtn.innerText = 'Share'
  root.appendChild(shareBtn)

  const shareBtnEl = document.getElementById('share-btn')
  shareBtnEl.addEventListener('click', () => {
    const spinner = document.createElement('span')
    const text = document.createElement('span')
    shareBtnEl.innerText = ''
    spinner.className = 'spinner-border spinner-border-sm me-1'
    text.innerText = 'Sharing...'
    shareBtnEl.appendChild(spinner)
    shareBtnEl.appendChild(text)

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
  stopBtn.className = 'btn btn-danger mt-3'
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
    message.innerText = `Running with PID: ${window.localStorage.getItem('micPid')}`
    message.hidden = false
    dropdown.hidden = true
    createStopBtn(root)
  } else {
    message.hidden = true
    dropdown.hidden = false
    createShareBtn(root)
  }
}

async function populateNodesList(response) {
  if (JSON.stringify(response) !== window.localStorage.getItem('nodesList')) {
    window.localStorage.setItem('nodesList', JSON.stringify(response))
    dropdown.innerHTML = null

    // const allDesktopAudioOption = document.createElement('option')
    // allDesktopAudioOption.innerText = ALL_DESKTOP_AUDIO_TEXT
    // allDesktopAudioOption.value = ALL_DESKTOP_AUDIO_TEXT
    // dropdown.appendChild(allDesktopAudioOption)

    for (const element of response) {
      const option = document.createElement('option')
      option.innerText = `${element.properties['media.name']} (${element.properties['application.name']})`
      option.value = element.properties['object.serial']
      dropdown.appendChild(option)
    }

    if (dropdown.innerHTML.indexOf('value="' + window.localStorage.getItem('selectedNode') + '"') > -1) {
      dropdown.value = window.localStorage.getItem('selectedNode')
    }

    selectedNode = dropdown.value
    dropdown.addEventListener('change', () => {
      selectedNode = dropdown.value
      window.localStorage.setItem('selectedNode', selectedNode)
    })
  }
}

function onReload(response) {
  populateNodesList(response)
}

function onResponse (response) {
  const root = document.getElementById('root')
  setInterval(() => {chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'GetNodes', args: [] }).then(onReload, onError)}, 1000)
  window.localStorage.setItem('nodesList', null)
  window.localStorage.setItem('selectedNode', null)
  populateNodesList(response)
  updateGui(root)
}

function onError (error) {
  console.error(error)
  message.innerText = 'The native connector is misconfigured or missing!'
  dropdown.hidden = true
}

chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'GetNodes', args: [] }).then(onResponse, onError)
