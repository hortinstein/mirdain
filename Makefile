.PHONY: dev dev-backend dev-frontend build-processors push-processors up down check

# ── Development ──────────────────────────────────────────────────────────────

dev: ## Run backend + frontend in dev mode (requires tmux or two terminals)
	@echo "Run 'make dev-backend' and 'make dev-frontend' in separate terminals"

dev-backend: ## Run backend with cargo watch
	cd backend && RUST_LOG=mirdain=debug cargo run

dev-frontend: ## Run Vite dev server (proxies /api + /ws to :3001)
	cd frontend && npm install && npm run dev

# ── Docker ───────────────────────────────────────────────────────────────────

up: ## Start everything with docker-compose
	docker compose up --build

down: ## Stop docker-compose services
	docker compose down

# ── Processor images ─────────────────────────────────────────────────────────

build-processors: ## Build all processor Docker images
	docker build -t mirdain/uppercase:latest processors/uppercase
	docker build -t mirdain/filter:latest    processors/filter
	docker build -t mirdain/merge:latest     processors/merge

push-processors: build-processors ## Push processor images (set REGISTRY env var)
	docker push $(REGISTRY)/mirdain/uppercase:latest
	docker push $(REGISTRY)/mirdain/filter:latest
	docker push $(REGISTRY)/mirdain/merge:latest

# ── Checks ───────────────────────────────────────────────────────────────────

check: ## Cargo check + clippy
	cargo check --workspace
	cargo clippy --workspace -- -D warnings
	cd frontend && npm run build

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
