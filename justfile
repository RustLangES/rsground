
dev:
	cd frontend && npm run dev &
	cd backend && mkdir -p .runners && cargo watch -x run
	|| pkill -9 webpack && pkill -9 cargo

dev-d:
	CARGO_TARGET_DIR=../.backend-output
	docker compose -f docker-compose.dev.yml up --build;
