import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
	plugins: [react()],
	server: {
		proxy: {
			"/api": {
				target: "http://localhost:5174",
				secure: false,
				changeOrigin: true,
				rewrite: (path: string) => path.replace(/^\/api/, '')
			}
		}
	}
})
