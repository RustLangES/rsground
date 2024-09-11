setup:
	@echo "Setting up the environment..."

dev:
	cd frontend && npm run dev &
	cd backend && mkdir -p .runners && cargo watch -x run
	|| pkill -9 webpack && pkill -9 cargo

dev-d:
    docker compose -f docker-compose.dev.yml up --build
