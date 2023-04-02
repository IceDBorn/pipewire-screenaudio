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

  document.getElementById('share-btn').addEventListener('click', () => {
    browser.runtime.sendNativeMessage('screenAudioMicConnector', { cmd: 'StartVirtmic', args: [{ node: dropdown.value }] })
  })

  document.getElementById('heading').innerText = 'Select audio node to share';
}

function onError (error) {
  console.error(error)
  document.getElementById('heading').innerText = 'The native connector is missing!';
  document.getElementById('share-btn').hidden = true;
  document.getElementById('dropdown').hidden = true;
}

// let sending = browser.runtime.sendNativeMessage("screenAudioMicConnector", { cmd: "StartVirtmic", args: [{ node: '' }] });
// let sending = browser.runtime.sendNativeMessage("screenAudioMicConnector", { cmd: "StopVirtmic", args: [{ micPid: 0 }] });
const sending = browser.runtime.sendNativeMessage('screenAudioMicConnector', { cmd: 'GetNodes', args: [] })
sending.then(onResponse, onError)
