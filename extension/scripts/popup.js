const MESSAGE_NAME = 'com.icedborn.pipewirescreenaudioconnector'
const EXT_VERSION = browser.runtime.getManifest().version

const dropdown = document.getElementById('dropdown')
const message = document.getElementById('message')
const buttonGroup = document.getElementById('btn-group')

let selectedNode = null
let nodesLoop = null

async function isRunning () {
  const micId = window.localStorage.getItem('micId')
  if (!micId) {
    return false
  }

  const { isRunning } = await chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'IsPipewireScreenAudioRunning', args: [{ micId }] })

  if (!isRunning) {
    window.localStorage.setItem('micId', null)
  }

  return isRunning
}

function createShareBtn (root) {
  if (document.getElementById('share-btn')) return
  const shareBtn = document.createElement('button')
  shareBtn.id = 'share-btn'
  shareBtn.className = 'btn btn-success me-2'
  shareBtn.innerText = 'Share'
  root.appendChild(shareBtn)
  const shareBtnEl = document.getElementById('share-btn')

  const eventListener = () => {
    shareBtnEl.removeEventListener('click', eventListener)
    const spinner = document.createElement('span')
    const text = document.createElement('span')
    shareBtnEl.innerText = ''
    spinner.className = 'spinner-border spinner-border-sm me-1'
    text.innerText = 'Sharing...'
    clearInterval(nodesLoop)
    shareBtnEl.appendChild(spinner)
    shareBtnEl.appendChild(text)
    document.getElementById('blacklist-btn').hidden = true

    chrome.runtime.sendMessage({ messageName: MESSAGE_NAME, message: 'node-shared', cmd: 'StartPipewireScreenAudio', args: [{ node: selectedNode }] })
  } 

  shareBtnEl.addEventListener('click', eventListener)
}

function createStopBtn (root) {
  if (document.getElementById('stop-btn')) return
  const stopBtn = document.createElement('button')
  stopBtn.id = 'stop-btn'
  stopBtn.className = 'btn btn-danger mt-3'
  stopBtn.innerText = 'Stop'
  root.appendChild(stopBtn)

  const stopBtnEl = document.getElementById('stop-btn')
  stopBtnEl.addEventListener('click', async () => {
    if (await isRunning()) {
      const micId = window.localStorage.getItem('micId')
      chrome.runtime.sendMessage({ messageName: MESSAGE_NAME, message: 'node-stopped', cmd: 'StopPipewireScreenAudio', args: [{ micId }] })
    }
  })
}

function createBlacklistBtn (root) {
  if (document.getElementById('blacklist-btn')) return
  const blacklistBtn = document.createElement('button')
  blacklistBtn.id = 'blacklist-btn'
  blacklistBtn.className = 'btn btn-danger px-3'
  blacklistBtn.innerText = 'Hide'
  root.appendChild(blacklistBtn)

  blacklistBtn.addEventListener('click', async () => {
    const nodesList = JSON.parse(window.localStorage.getItem('nodesList'))
    const nodeToBlacklist = { name: nodesList.find(n => n.properties['object.serial'] === dropdown.value).properties['application.name'] }
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
    message.innerText = `Running virtmic Id: ${window.localStorage.getItem('micId')}`
    message.hidden = false
    dropdown.hidden = true
    createStopBtn(buttonGroup)
  } else if (dropdown.children.length) {
    message.hidden = true
    dropdown.hidden = false
    createShareBtn(buttonGroup)
    createBlacklistBtn(buttonGroup)
  } else {
    message.innerText = 'No nodes available to share...'
    message.className = 'mt-5'
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
      document.getElementById('share-btn').hidden = true
      document.getElementById('blacklist-btn').hidden = true
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
  window.localStorage.setItem('selectedNode', null)
  updateGui()
}

function onError (error) {
  console.error(error)
  message.innerText = 'The native connector is misconfigured or missing!'
  dropdown.hidden = true
}

function handleMessage (message) {
  if (message === 'mic-id-updated') {
    const shareBtnEl = document.getElementById('share-btn')
    const hideBtnEl = document.getElementById('blacklist-btn')
    buttonGroup.removeChild(shareBtnEl)
    buttonGroup.removeChild(hideBtnEl)
    updateGui()
  }

  if (message === 'mic-id-removed') {
    const stopBtnEl = document.getElementById('stop-btn')
    buttonGroup.removeChild(stopBtnEl)
    updateGui()
  }
}

chrome.runtime.onMessage.addListener(handleMessage)

chrome.runtime.sendNativeMessage(MESSAGE_NAME, { cmd: 'GetVersion', args: [] }).then(onResponse, onError)

