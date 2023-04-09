const table = document.getElementById('blacklisted')
const remove = document.getElementById('remove-btn')
const clear = document.getElementById('clear-btn')
const placeholder = document.getElementById('placeholder')
const tableInitialHTML = table.innerHTML

function addItem (item) {
  const tr = document.createElement('tr')
  const th = document.createElement('th')
  const div = document.createElement('div')
  const input = document.createElement('input')
  const label = document.createElement('label')

  div.className = 'form-check'
  th.scope = 'row'
  input.className = 'form-check-input'
  input.type = 'checkbox'
  input.id = 'item'
  label.className = 'form-check-label'
  label.htmlFor = 'item'
  label.innerText = item

  div.appendChild(input)
  div.appendChild(label)

  th.appendChild(div)

  tr.appendChild(th)

  table.appendChild(tr)

  placeholder.hidden = true
}

function populateBlacklistedList () {
  for (const item of JSON.parse(window.localStorage.getItem('blacklistedNodes') || '[]')) {
    addItem(item.name)
  }
}

remove.addEventListener('click', () => {
  let items = Array.from(table.children)

  for (const item of items) {
    const children = item.firstElementChild.firstElementChild
    if (children) {
      if (children.firstElementChild.checked) {
        let blacklistedNodes = JSON.parse(window.localStorage.getItem('blacklistedNodes'))
        blacklistedNodes = blacklistedNodes.filter(node => node.name !== children.lastElementChild.innerText)
        table.removeChild(item)
        items = Array.from(table.children)
        window.localStorage.setItem('blacklistedNodes', JSON.stringify(blacklistedNodes))
      }
      if (items.length <= 1) {
        placeholder.hidden = false
      }
    }
  }
})

clear.addEventListener('click', () => {
  if (window.confirm('Remove all blacklisted nodes?')) {
    window.localStorage.setItem('blacklistedNodes', [])
    table.innerHTML = tableInitialHTML
  }
})

// Repopulate table upon hiding a node
function handleMessage(message) {
  if (message === 'node-hidden') {
    table.innerHTML = null
    populateBlacklistedList()
  }
}

chrome.runtime.onMessage.addListener(handleMessage);

populateBlacklistedList()
