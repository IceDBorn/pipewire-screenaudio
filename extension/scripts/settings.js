const table = document.getElementById('blacklisted')
const remove = document.getElementById('remove-btn')
const placeholder = document.getElementById('placeholder')

function addItem(item) {
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

// for (const item of blacklistedNodes) {
//   addItem(item)
// }

remove.addEventListener('click', () => {
  let items = Array.from(table.children)

  for (const item of items) {
    let checked = false
    const children = item.firstElementChild.firstElementChild
    if (children) {
      checked = children.firstElementChild.checked

      if (checked) {
        table.removeChild(item)
        items = Array.from(table.children)
      }
      if (items.length <= 1) {
        placeholder.hidden = false
      }
    }
  }
})

addItem('Item 1')
addItem('Item 2')
