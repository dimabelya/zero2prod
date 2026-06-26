# zero2prod

A production-ready email newsletter API built in Rust, following
[Zero To Production In Rust](https://www.zero2prod.com/) by Luca Palmieri.

This is a learning project. The goal is not just to build a newsletter service —
it is to understand what production backend development actually looks like:
async runtimes, web frameworks, database integration, testing strategy, and
the systems concepts that tie them together.

---

## What It Does

Exposes an HTTP API that allows visitors to subscribe to an email newsletter.
Subscribers provide their name and email via a form. The API validates the input
and (eventually) stores it in a PostgreSQL database and sends a confirmation email.

**Current endpoints:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health_check` | Returns 200 OK if the server is running |
| POST | `/subscriptions` | Accepts a name and email to subscribe |

---

## Running Locally

**Prerequisites:**
- Rust (stable) — install via [rustup.rs](https://rustup.rs)
- Docker
- sqlx-cli

```bash
cargo install --version='~0.8' sqlx-cli --no-default-features --features rustls,postgres
```

**Start the database:**

```bash
./scripts/init_db.sh
```

**Run the application:**

```bash
cargo run
```

**Run the tests:**

```bash
cargo test
```

**Check a specific endpoint manually:**

```bash
curl http://127.0.0.1:8000/health_check

curl -X POST http://127.0.0.1:8000/subscriptions \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "name=Ursula%20Le%20Guin&email=ursula@example.com"
```

---

## Project Status

Working through the book chapter by chapter. Each section below tracks what
has been built and what is coming next.

- [x] HTTP server with actix-web
- [x] Health check endpoint
- [x] Form parsing and validation (name + email required)
- [x] Integration test suite with black-box testing approach
- [x] PostgreSQL running in Docker
- [x] Database schema and migrations with sqlx-cli
- [x] Least-privilege database user setup
- [ ] Actually storing subscribers in the database (sqlx queries)
- [ ] Configuration management
- [ ] Confirmation emails
- [ ] Deployment

---

## Tech Stack

| Technology | Role | Why |
|------------|------|-----|
| Rust | Language | Memory safety without a garbage collector, strong type system |
| Tokio | Async runtime | Handles concurrent I/O without blocking threads |
| actix-web | Web framework | Production-proven, runs on Tokio, large ecosystem |
| serde | Serialization | Compile-time (de)serialization for any format |
| sqlx | Database driver | Async, compile-time SQL verification against a live DB |
| PostgreSQL | Database | Reliable relational DB, excellent Rust ecosystem support |
| Docker | Local DB environment | Reproducible setup that mirrors production |
| sqlx-cli | Migration tool | Versioned, repeatable schema changes |
| reqwest | HTTP client (tests only) | Used in integration tests to call the running API |

---

## Architecture

### Binary vs Library Split

The project is structured as both a library (`src/lib.rs`) and a binary
(`src/main.rs`). This is deliberate.

Rust integration tests live in `tests/` and can only import from library crates,
not binaries. By putting all application logic in `lib.rs`, the test suite can
call `zero2prod::run()` directly, spinning up real instances of the server.
`main.rs` is just a thin wrapper:

```
main.rs  →  calls zero2prod::run()  →  lib.rs contains everything
```

### Request Lifecycle

```
Browser / curl
     │
     │  HTTP request
     ▼
TcpListener (binds a port, hands connection to actix-web)
     │
     ▼
HttpServer (one worker thread per CPU core)
     │
     ▼
App (routing table — matches path + method to handler)
     │
     ├── GET  /health_check  →  health_check()  →  200 OK
     │
     └── POST /subscriptions →  subscribe()
                                    │
                                    ▼
                               Form<FormData> extractor
                               (serde parses URL-encoded body)
                                    │
                               valid? → 200 OK
                               missing field? → 400 Bad Request (automatic)
```

### Why Tokio and not actix-web's own async runtime?

Rust does not ship an async runtime in its standard library — you have to bring
one in as a dependency. This was a deliberate language design choice: different
use cases have different needs (embedded systems, game servers, web APIs), and no
single runtime fits all of them.

Tokio is the most widely adopted async runtime in the Rust ecosystem. It uses a
work-stealing thread pool — each CPU core gets a thread, and idle threads can
steal tasks from busy ones to keep all cores occupied.

actix-web runs on top of Tokio rather than providing its own runtime. This
matters because any other library that also requires Tokio (database drivers,
HTTP clients, etc.) will integrate without friction. If actix-web used its own
runtime, every dependency that assumed Tokio would need a compatibility shim or
would outright not work. By standardizing on Tokio, the entire async Rust
ecosystem can interoperate cleanly.

`#[tokio::main]` bootstraps the runtime so an `async fn main()` can exist.
Every `.await` in the code is a yield point — the task hands control back to
Tokio, which can then make progress on something else.

### Why actix-web?

actix-web handles the low-level HTTP work so application code only has to deal
with business logic. It provides:

- Routing (match a path + HTTP method to a handler function)
- Extractors (automatically parse request data into Rust types)
- Middleware support
- Worker-per-core threading model on top of Tokio

The extractor system is what makes `web::Form<FormData>` work. actix-web sees
the `Form<FormData>` argument on the handler, calls serde to deserialize the
request body into `FormData`, and returns a `400 Bad Request` automatically if
any required field is missing. The handler never runs if the data is invalid.

### Why serde?

serde is a framework for serialization and deserialization in Rust. It does not
implement any specific format itself — instead it defines traits (`Serialize`,
`Deserialize`) that other crates implement for their formats (JSON, URL-encoded
forms, YAML, etc.).

`#[derive(serde::Deserialize)]` on `FormData` auto-generates the deserialization
logic at compile time. No reflection, no runtime overhead.

---

## Database

### Why PostgreSQL?

PostgreSQL is a battle-tested relational database with excellent Rust ecosystem
support, easy Docker setup, and solid cloud provider managed offerings. For a
backend API that needs to handle concurrent requests reliably, a relational
database with transactions is the right default.

### Why sqlx?

sqlx is an async database driver that verifies SQL queries at compile time by
connecting to a real database during `cargo build`. Typos in column names or
wrong types become compile errors, not runtime surprises.

This is different from most database libraries in other languages which only
catch SQL errors at runtime.

### Schema

```sql
CREATE TABLE subscriptions (
    id            uuid        NOT NULL PRIMARY KEY,
    email         TEXT        NOT NULL UNIQUE,
    name          TEXT        NOT NULL,
    subscribed_at timestamptz NOT NULL
);
```

- `uuid` for the primary key — no business meaning, no collision risk
- `UNIQUE` on email — enforced at the database level, not just the application
- `timestamptz` — timezone-aware timestamp
- `NOT NULL` on all fields — invalid state is unrepresentable at the DB level

### Migrations

Schema changes are managed with `sqlx migrate`. Each migration is a `.sql` file
in `migrations/` with a timestamp prefix. sqlx tracks which migrations have been
applied in a `_sqlx_migrations` table, making schema evolution versioned and
reproducible across environments.

```
migrations/
└── 20260623012553_create_subscriptions_table.sql
```

---

## Database Setup (Security Design)

The `scripts/init_db.sh` script sets up PostgreSQL with two separate users:

| User | Name | Privileges | Used By |
|------|------|-----------|---------|
| Superuser | `postgres` | Full admin | Setup only |
| App user | `app` | CREATEDB only | Application + migrations |

The running application connects as `app`, not `postgres`. This is the principle
of least privilege — if the application is ever compromised, the attacker only
has `app`'s permissions.

**What the script does, in order:**

1. Checks that `sqlx-cli` is installed
2. Sets default config values (all overridable via environment variables)
3. Starts a new postgres Docker container with Docker health checks
4. Waits until the container reports healthy via `pg_isready`
5. Creates the `app` user and grants it `CREATEDB`
6. Runs `sqlx database create` (creates the `newsletter` database)
7. Runs `sqlx migrate run` (applies all pending migrations)

**Environment variables (all optional, defaults shown):**

```bash
POSTGRES_PORT=5432
SUPERUSER=postgres
SUPERUSER_PWD=password
APP_USER=app
APP_USER_PWD=secret
APP_DB_NAME=newsletter
SKIP_DOCKER=true   # set this to skip Docker if postgres is already running
```

---

## Testing Strategy

Tests live in `tests/health_check.rs` and use a **black-box approach**: each test
spins up a real instance of the application on a random OS-assigned port, then
interacts with it using HTTP calls exactly as a real client would.

This means tests are decoupled from implementation details. Switching the web
framework or reorganizing the code would not require rewriting the tests — only
the behavior matters.

**`spawn_app()` — how tests start the server:**

```rust
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0") // port 0 = OS picks a free port
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server); // run server as background task
    format!("http://127.0.0.1:{}", port)
}
```

Port 0 is special — the OS assigns any free port. This lets multiple tests run
in parallel without conflicts. `tokio::spawn` runs the server as a background
task so the test can proceed to make requests against it.

**Test cases:**

- `health_check_works` — GET /health_check returns 200 with empty body
- `subscribe_returns_a_200_for_valid_form_data` — valid name + email returns 200
- `subscribe_returns_a_400_when_data_is_missing` — missing fields return 400

---

## Project Structure

```
zero2prod/
├── migrations/
│   └── 20260623012553_create_subscriptions_table.sql
├── scripts/
│   └── init_db.sh          # sets up Docker + Postgres + migrations
├── src/
│   ├── lib.rs              # all application logic (importable by tests)
│   └── main.rs             # binary entrypoint, calls lib::run()
├── tests/
│   └── health_check.rs     # black-box integration tests
├── Cargo.toml
└── Cargo.lock
```

---

## Systems Concepts Demonstrated

**Async I/O** — Tokio runtime, cooperative scheduling, yield points via `.await`

**HTTP server architecture** — worker threads, routing, extractors, middleware

**Type-driven design** — using Rust's type system to make invalid states
unrepresentable (missing form fields caught at deserialization, not in handler logic)

**Black-box integration testing** — testing via the public API surface rather
than calling internal functions directly

**Database migrations** — versioned, reproducible schema evolution

**Principle of least privilege** — separate superuser and application database users

**Port binding and OS networking** — TcpListener, port 0 for random port assignment

---

## Reference

- [Zero To Production In Rust](https://www.zero2prod.com/) by Luca Palmieri
- [actix-web docs](https://actix.rs/docs/)
- [sqlx docs](https://docs.rs/sqlx)
- [Tokio docs](https://tokio.rs)
- [serde docs](https://serde.rs)
