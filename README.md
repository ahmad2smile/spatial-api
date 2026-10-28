# Spatial



## Run

Requires Docker

1. Database

```bash
docker compose up
```

2. Migartion:

```bash
export DATABASE_URL="postgres://postgres:password@localhost/spatial"
sqlx database create
sqlx migrate run
```
3. Api run:

```bash
export RUST_LOG="spatial=debug"
cargo run
```

4. Open Docs:

[http://localhost:3000/swagger-ui](http://localhost:3000/swagger-ui)

## Test

Requires Docker

```bash
cargo test
```
