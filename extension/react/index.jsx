import { createRoot } from 'react-dom/client'
import Button from './components/button'

function App () {
  return (
    <div>
      <Button />
    </div>
  )
}

const rootEl = document.getElementById('root')
const root = createRoot(rootEl)
root.render(<App />)
