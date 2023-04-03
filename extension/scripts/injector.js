const nullthrows = (v) => {
  if (v == null) throw new Error('null')
  return v
}

function injectCode (src) {
  const script = document.createElement('script')
  script.src = src
  script.onload = function () {
    console.log('pipewire-screenaudio script injected')
    this.remove()
  }

  nullthrows(document.head || document.documentElement).appendChild(script)
}

injectCode(chrome.runtime.getURL('/scripts/index.js'))
