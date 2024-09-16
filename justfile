
dev:
	USER=$(whoami) \
	docker compose -f ./infra/docker-compose.dev.yml up --build;
