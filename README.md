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
curl -v http://127.0.0.1:8000/health_check

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

---

## Reference

- [Zero To Production In Rust](https://www.zero2prod.com/) by Luca Palmieri
- [actix-web docs](https://actix.rs/docs/)
- [sqlx docs](https://docs.rs/sqlx)
- [Tokio docs](https://tokio.rs)
- [serde docs](https://serde.rs)
