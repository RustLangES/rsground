import { ReactElement, StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import StatusBar from './components/status-bar';
import './index.css'

function App(): ReactElement {

	return <>
		<StatusBar 
			github={{text: undefined, loading: false}} 
			insights={{errors: 0, warnings: 0}} 
			container={{ text: undefined, loading: false}} 
		/>
	</>;
}

createRoot(document.getElementById('root')!)
	.render(<StrictMode><App /></StrictMode>)
