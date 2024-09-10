setup:
	@echo "Setting up the environment..."

dev:
	cd frontend && npm run dev &
	cd backend && mkdir -p .runners && cargo watch -x run
	|| pkill -9 webpack && pkill -9 cargo

compose file:
    @echo "Using Docker Compose file: {{file}}"
    docker compose -f infra/{{file}} up --build
