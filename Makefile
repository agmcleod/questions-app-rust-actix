SHELL := /bin/bash

db_url := postgres://postgres:my_password@localhost:5434/my_database

test_prepare:
	DATABASE_URL=postgres://root@localhost:5433/my_database_test diesel migration run --migration-dir=db/migrations

test:
	docker-compose -f docker-compose.test.yml exec database_test psql -d my_database_test --c="TRUNCATE questions"
	DATABASE_URL=postgres://root@localhost:5433/my_database_test \
		cargo test $(T) -- --nocapture --test-threads=1

run_server:
	CLIENT_HOST=http://localhost:3000 \
		RUST_BACKTRACE=full \
		cargo run --bin server

create_migration:
	DATABASE_URL=$(db_url) diesel migration generate $(name) --migration-dir=db/migrations

migrate:
	DATABASE_URL=$(db_url) diesel migration run --migration-dir=db/migrations

redo_migrate:
	DATABASE_URL=$(db_url) diesel migration redo --migration-dir=db/migrations

.PHONY: run_server test test_prepare
