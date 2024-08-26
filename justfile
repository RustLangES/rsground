
setup:


dev:
	cd frontend && npm run dev &
	cd backend && cargo watch -x run
	|| pkill -9 webpack && pkill -9 cargo
