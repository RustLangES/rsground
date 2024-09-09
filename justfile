
setup:


dev:
	cd frontend && npm run dev &
	cd backend && mkdir -p .runners && cargo watch -x run
	|| pkill -9 webpack && pkill -9 cargo

compose_dev:
	cd infra && docker-compose -f compose.dev.yml up -d --build

compose_prod:
	cd infra && docker-compose -f compose.prod.yml up -d
