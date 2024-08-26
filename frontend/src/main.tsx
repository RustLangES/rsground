import { ReactElement, StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'

function App(): ReactElement {
	return <></>;
}

createRoot(document.getElementById('root')!)
	.render(<StrictMode><App /></StrictMode>)
