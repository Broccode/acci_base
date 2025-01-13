.PHONY: dev test prod clean env-dev env-test env-prod migrate migrate-down migrate-status

# Default target
dev: env-dev
	docker compose -f docker/compose.dev.yaml up

# Development environment
env-dev:
	./docker/compose-env.sh --config config/config.dev.toml --env docker/.env

dev-build: env-dev
	docker compose -f docker/compose.dev.yaml build

dev-down:
	docker compose -f docker/compose.dev.yaml down -v

# Test environment
test: env-test
	docker compose -f docker/compose.test.yaml up --abort-on-container-exit --exit-code-from test-runner

env-test:
	./docker/compose-env.sh --config config/config.test.toml --env docker/.env

test-build: env-test
	docker compose -f docker/compose.test.yaml build

test-down:
	docker compose -f docker/compose.test.yaml down -v

# Production environment
prod: env-prod
	docker compose -f docker/compose.prod.yaml up -d

env-prod:
	./docker/compose-env.sh --config config/config.prod.toml --env docker/.env

prod-build: env-prod
	docker compose -f docker/compose.prod.yaml build

prod-down:
	docker compose -f docker/compose.prod.yaml down -v

# Utility targets
clean: dev-down test-down prod-down
	rm -f docker/.env

logs:
	docker compose -f docker/compose.$(ENV).yaml logs -f

ps:
	docker compose -f docker/compose.$(ENV).yaml ps

# Migration targets
migrate: ## Run database migrations
	cargo run --package migration

migrate-down: ## Revert last database migration
	cargo run --package migration -- down

migrate-status: ## Show migration status
	cargo run --package migration -- status

# Help target
help:
	@echo "Available targets:"
	@echo "  dev         - Start development environment"
	@echo "  dev-build   - Build development containers"
	@echo "  dev-down    - Stop and remove development environment"
	@echo "  test        - Run tests"
	@echo "  test-build  - Build test containers"
	@echo "  test-down   - Stop and remove test environment"
	@echo "  prod        - Start production environment"
	@echo "  prod-build  - Build production containers"
	@echo "  prod-down   - Stop and remove production environment"
	@echo "  clean       - Clean up all environments"
	@echo "  logs        - Show logs (use ENV=dev|test|prod)"
	@echo "  ps          - Show running containers (use ENV=dev|test|prod)"
	@echo "  migrate     - Run database migrations"
	@echo "  migrate-down - Revert last database migration"
	@echo "  migrate-status - Show migration status"
