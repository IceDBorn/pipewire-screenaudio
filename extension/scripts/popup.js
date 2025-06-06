const MESSAGE_NAME = 'com.icedborn.pipewirescreenaudioconnector'
const EXT_VERSION = browser.runtime.getManifest().version

const dropdown = document.getElementById('dropdown')
const message = document.getElementById('message')
const buttonGroup = document.getElementById('btn-group')
const shareStopBtn = document.getElementById('share-stop-btn')
let shareStopBtnState = null
let nodesLoop = null

dropdown.addEventListener('change', () => {
  setSelectedNode(dropdown.value)
  chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'SetSharingNode', args: [{ node: selectedNode, micId }] })
})

let selectedNode = null
function setSelectedNode (id) {
  selectedNode = id
  window.localStorage.setItem('selectedNode', id)

  const blacklistBtn = document.getElementById('blacklist-btn')
  if (blacklistBtn) {
    blacklistBtn.disabled = shouldDisableBlacklistBtn();
  }
}

let micId = null
function setMicId (id, skipStorage) {
  micId = JSON.parse(id)
  skipStorage || window.localStorage.setItem('micId', micId)
}

function shouldDisableBlacklistBtn () {
  // Disable on All Desktop Audio
  if (!selectedNode) return false
  return selectedNode.toString() === '-1'
}

function enqueueCommandToBackground (command) {
  chrome.runtime.sendMessage({ messageName: MESSAGE_NAME, message: 'enqueue-command', command })
}

async function isRunning () {
  setMicId(window.localStorage.getItem('micId'), true)

  if (!micId) {
    return false
  }

  const { isRunning } = await chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'IsPipewireScreenAudioRunning', args: [{ micId }] })

  if (!isRunning) {
    setMicId(null)
  }

  return isRunning
}

function setButtonToShare() {
  shareStopBtnState = "share"
  shareStopBtn.className = 'btn btn-success me-2'
  shareStopBtn.innerText = 'Share'

  const eventListener = () => {
    shareStopBtn.removeEventListener('click', eventListener)
    shareStopBtnState = "sharing"
    const spinner = document.createElement('span')
    const text = document.createElement('span')
    shareStopBtn.innerText = ''
    spinner.className = 'spinner-border spinner-border-sm me-1'
    text.innerText = 'Sharing...'
    shareStopBtn.appendChild(spinner)
    shareStopBtn.appendChild(text)
    if (document.getElementById('blacklist-btn')) {
      document.getElementById('blacklist-btn').hidden = true
    }

    enqueueCommandToBackground({
      cmd: 'StartPipewireScreenAudio',
      maps: { outMap: [[ 'micId', 'micId' ]] }, // Set the `micId` in LocalStorage to the incoming `micId`
      event: 'mic-id-updated'
    })

    enqueueCommandToBackground({
      cmd: 'SetSharingNode',
      args: { node: selectedNode },
      maps: { inMap: [[ 'micId', 'micId' ]] } // Read the `micId` from LocalStorage and pass it as the `micId` arg
    })
  }

  shareStopBtn.addEventListener('click', eventListener)
}


function setButtonToStop() {
  shareStopBtnState = "stop"
  shareStopBtn.className = 'btn btn-danger'
  shareStopBtn.innerText = 'Stop'

  const eventListener = async () => {
    shareStopBtn.removeEventListener('click', eventListener)
    if (await isRunning()) {
      enqueueCommandToBackground({
        cmd: 'StopPipewireScreenAudio',
        args: { micId },
        maps: { outMap: [[ 'micId', null ]] }, // Clear the `micId` in LocalStorage
        event: 'mic-id-removed'
      })
    }
  };

  shareStopBtn.addEventListener('click', eventListener)
}

function createBlacklistBtn (root) {
  if (document.getElementById('blacklist-btn')) return
  const blacklistBtn = document.createElement('button')
  blacklistBtn.id = 'blacklist-btn'
  blacklistBtn.className = 'btn btn-danger px-3'
  blacklistBtn.innerText = 'Hide'
  blacklistBtn.disabled = shouldDisableBlacklistBtn()
  root.appendChild(blacklistBtn)

  blacklistBtn.addEventListener('click', async () => {
    const nodesList = JSON.parse(window.localStorage.getItem('nodesList'))
    const nodeToBlacklist = { name: nodesList.find(n => n.properties['object.serial'] === parseInt(dropdown.value)).properties['application.name'] }
    const blacklistedNodes = []

    const items = window.localStorage.getItem('blacklistedNodes')
    if (items) {
      blacklistedNodes.push(...JSON.parse(items))
    }

    blacklistedNodes.push(nodeToBlacklist)
    window.localStorage.setItem('blacklistedNodes', JSON.stringify(blacklistedNodes))
    window.localStorage.setItem('nodesList', null)
    chrome.runtime.sendMessage('node-hidden')
  })
}

async function updateGui () {
  if (await isRunning()) {
    message.innerText = `Running virtmic Id: ${micId}`
    message.hidden = false
    dropdown.hidden = false
    shareStopBtn.hidden = false
    if (shareStopBtnState !== "stop")
      setButtonToStop()
  } else if (dropdown.children.length) {
    message.hidden = true
    dropdown.hidden = false
    shareStopBtn.hidden = false
    if (shareStopBtnState !== "share")
      setButtonToShare()
    createBlacklistBtn(buttonGroup)
  } else {
    message.innerText = 'No nodes available to share...'
    shareStopBtn.hidden = true
    message.hidden = false
    dropdown.hidden = true
  }
}

async function populateNodesList (response) {
  if (JSON.stringify(response) !== window.localStorage.getItem('nodesList')) {
    let whitelistedNodes = [...response]
    window.localStorage.setItem('nodesList', JSON.stringify(response))
    dropdown.innerHTML = null

    const blacklistedNodes = window.localStorage.getItem('blacklistedNodes')

    if (blacklistedNodes?.length) {
      const bnNames = JSON.parse(blacklistedNodes).map(bn => bn.name)
      whitelistedNodes = response.filter(node => !bnNames.includes(node.properties['application.name']))
    }

    // If last selected node doesn't exist in whitelistedNodes, ignore it
    if (
      !whitelistedNodes
        .map(element => element.properties['object.serial'])
        .includes(parseInt(window.localStorage.getItem('selectedNode')))
    ) {
      setSelectedNode(null);
    }

    for (const element of whitelistedNodes) {
      let text = element.properties['media.name']
      if (element.properties['application.name']) {
        text += ` (${element.properties['application.name']})`
      }

      const option = document.createElement('option')
      option.innerText = text
      option.value = element.properties['object.serial']

      dropdown.appendChild(option)
    }

    if (!dropdown.children.length) {
      message.innerText = 'No nodes available to share...'
      message.className = 'mt-5'
      message.hidden = false
      dropdown.hidden = true
      document.getElementById('share-stop-btn').hidden = true
      document.getElementById('blacklist-btn').hidden = true
    }

    if (dropdown.innerHTML.indexOf('value="' + window.localStorage.getItem('selectedNode') + '"') > -1) {
      dropdown.value = window.localStorage.getItem('selectedNode')
    }

    selectedNode = dropdown.value
  }
}

function checkVersionMatch (nativeVersion) {
  const extVersionSplit = EXT_VERSION.split('.')
  const nativeVersionSplit = nativeVersion.split('.')
  return extVersionSplit[0] === nativeVersionSplit[0] && extVersionSplit[1] === nativeVersionSplit[1]
}

function onReload (response) {
  populateNodesList(response)
  updateGui()
}

function onResponse (response) {
  if (!checkVersionMatch(response.version)) {
    message.innerText = `Version mismatch\nExtension: ${EXT_VERSION}\nNative: ${response.version}`
    dropdown.hidden = true
    return
  }
  const settings = document.getElementById('settings')
  settings.addEventListener('click', async () => {
    window.open('settings.html')
  })
  chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'GetNodes', args: [] }).then(onReload, onError)
  nodesLoop = setInterval(() => { chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'GetNodes', args: [] }).then(onReload, onError) }, 1000)
  window.localStorage.setItem('nodesList', null)
  updateGui()
}

function onError (error) {
  console.error(error)
  message.innerText = 'The native connector is misconfigured or missing!'
  dropdown.hidden = true
}

function handleMessage (message) {
  if (message === 'mic-id-updated') {
    setMicId(window.localStorage.getItem('micId'), true)

    const blacklistBtn = document.getElementById('blacklist-btn')
    buttonGroup.removeChild(blacklistBtn)
    updateGui()
  }

  if (message === 'mic-id-removed') {
    updateGui()
  }
}

chrome.runtime.onMessage.addListener(handleMessage)

chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'GetVersion', args: [] }).then(onResponse, onError)

