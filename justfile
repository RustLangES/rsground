
dev:
	cd frontend && npm run dev &
	cd backend && mkdir -p .runners && cargo watch -x run
	|| pkill -9 webpack && pkill -9 cargo

dev-d:
	if ! which cargo-watch > /dev/null; then cargo install cargo-watch; fi;
	mkdir -p ./.build-context;
	cp "$(which cargo-watch)" ./.build-context/cargo-watch;
	docker compose -f docker-compose.dev.yml up --build;
