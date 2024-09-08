import { ReactElement, StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import StatusBar from './components/status-bar';

import './index.css'


function App(): ReactElement {
	

	return <>
		<StatusBar />
	</>;
}

createRoot(document.getElementById('root')!)
	.render(<StrictMode><App /></StrictMode>)
