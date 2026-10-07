# Task Management App — Rust API + React

A small full-stack task manager:

- **Backend** (`backend/`): Rust · Axum 0.8 · SQLx (PostgreSQL) · Redis · JWT · Argon2 · email-based 2FA · role-based access · OpenAPI/Swagger
- **Frontend** (`frontend/`): React 18 · TypeScript · Vite · React Router

The main validation point is `GET /tasks/view-my-tasks` as James Bond. It returns exactly 3 tasks with `cache.hit = false` on the first call and `cache.hit = true` on the second, and the frontend shows the same three tasks.

![My tasks as James Bond](docs/screenshots/03-staff-my-tasks-403.png)

---

## Contents

1. [Prerequisites](#prerequisites)
2. [Quick start](#quick-start)
3. [Backend setup](#backend-setup)
4. [Frontend setup](#frontend-setup)
5. [Validation workflow](#validation-workflow)
6. [Final validation response](#final-validation-response)
7. [Tests](#tests)
8. [API reference](#api-reference)
9. [Design notes](#design-notes)
10. [Project structure](#project-structure)
11. [Screenshots](#screenshots)

## Prerequisites

| Tool | Version used |
|---|---|
| Rust (stable) | 1.85+ (developed on 1.98) |
| Node.js | 20+ (developed on 24) |
| Docker + Docker Compose | for PostgreSQL 16 and Redis 7 |
| `sqlx-cli` (optional) | `cargo install sqlx-cli --no-default-features --features postgres,rustls` |

## Quick start

```bash
# 1. Infrastructure (Postgres + Redis)
docker compose up -d

# 2. Backend (applies migrations automatically on startup)
cp .env.example backend/.env
cd backend && cargo run          # http://127.0.0.1:8080, Swagger at /swagger-ui

# 3. Frontend (new terminal)
cd frontend && npm install && npm run dev   # http://localhost:5173

# 4. Run the whole validation flow against the API (new terminal)
./scripts/validate.sh            # bash: needs curl + jq
./scripts/validate.ps1           # Windows PowerShell
```

## Backend setup

```bash
docker compose up -d              # postgres:5432 (taskapp/taskapp), redis:6379
cp .env.example backend/.env      # adjust secrets if you like
cd backend
```

**Migrations** live in `backend/migrations/`. They run automatically when the server starts (`sqlx::migrate!`), or you can run them manually:

```bash
sqlx migrate run                  # uses DATABASE_URL from backend/.env
```

**Run:**

```bash
cargo run
```

**Seed** the two validation users (dev only):

```bash
curl -X POST http://127.0.0.1:8080/seed/users                # create/refresh Admin and James Bond
curl -X POST "http://127.0.0.1:8080/seed/users?reset=true"   # also wipe tasks, challenges, email logs and the cache
```

| User | Email | Password | Role |
|---|---|---|---|
| Admin | `admin@example.com` | `Admin@123` | admin |
| James Bond | `jamesbond@example.com` | `Bond@007` | staff |

### Environment variables

See [`.env.example`](.env.example). The important ones:

| Variable | Default | Purpose |
|---|---|---|
| `DATABASE_URL` | — | Postgres connection string |
| `REDIS_URL` | `redis://localhost:6379` | Redis for the task cache |
| `JWT_SECRET` / `OTP_SECRET` | — | HMAC keys (≥ 32 chars) for JWTs and 2FA code hashing |
| `APP_ENV` | `development` | `development` enables `/seed/users` and `/dev/email-logs/latest` |
| `JWT_TTL_MINUTES` | `60` | Access token lifetime |
| `OTP_TTL_SECONDS` | `300` | 2FA code lifetime (5 minutes) |
| `OTP_MAX_ATTEMPTS` | `5` | Wrong codes allowed per challenge |
| `CACHE_TTL_SECONDS` | `300` | Redis TTL for cached task lists |
| `CORS_ORIGINS` | `http://localhost:5173` | Allowed browser origins |

## Frontend setup

```bash
cd frontend
npm install
npm run dev        # http://localhost:5173
```

The UI calls `/api/*`, which the Vite dev server proxies to `http://127.0.0.1:8080` (see `vite.config.ts`), so you don't need any CORS setup. `frontend/.env.example` documents how to point it somewhere else.

**In the UI:**

1. Pick **Admin**, then **Continue**. The API creates a 2FA challenge.
2. Click **Fetch code from dev mailbox**, which reads `GET /dev/email-logs/latest` (or copy the code from the backend console), then click **Verify and sign in**.
3. On **Admin**, create 5 tasks, tick 3 of them, and click **Assign 3 selected** to James Bond.
4. **Log out**, then sign in as **James Bond** the same way.
5. **My tasks** shows the 3 assigned tasks with a `cache.hit = false` badge. **Refresh** changes it to `cache.hit = true`.
6. **Create task** on the same page shows the 403 *Access denied* message.

## Validation workflow

`scripts/validate.sh` and `scripts/validate.ps1` automate all of the steps below and assert on each result. To do it by hand with curl:

```bash
API=http://127.0.0.1:8080

# 1. Create users Admin and James Bond
curl -X POST "$API/seed/users?reset=true"

# 2. Start login as Admin -> returns login_challenge_id, NOT a JWT
curl -X POST $API/auth/login -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"Admin@123"}'
# {"login_challenge_id":"…","two_factor_required":true,"expires_at":"…","message":"A verification code was sent to a****@example.com. …"}

# 3. Read the verification code (dev email log; it's also printed in the backend console)
curl "$API/dev/email-logs/latest?email=admin@example.com"
# {"to":"admin@example.com","subject":"…","body":"Your verification code is 123456 …","code":"123456","login_challenge_id":"…","sent_at":"…"}

# 4. Verify 2FA -> Admin JWT
curl -X POST $API/auth/verify-2fa -H 'Content-Type: application/json' \
  -d '{"login_challenge_id":"<id>","code":"<code>"}'
# {"access_token":"eyJ…","token_type":"Bearer","expires_in":3600,"user":{…,"role":"admin"}}
ADMIN=<access_token>

# 5. Create exactly 5 tasks (repeat with different titles/priorities)
curl -X POST $API/tasks -H "Authorization: Bearer $ADMIN" -H 'Content-Type: application/json' \
  -d '{"title":"Infiltrate SPECTRE meeting","description":"Rome","priority":"high"}'

# 6. Assign exactly 3 of them to James Bond
curl -X POST $API/tasks/assign -H "Authorization: Bearer $ADMIN" -H 'Content-Type: application/json' \
  -d '{"task_ids":["<id1>","<id2>","<id3>"],"assignee_email":"jamesbond@example.com"}'

# 7-8. Log in as James Bond (same as steps 2-4) -> BOND=<access_token>

# 9. James Bond tries to create a task -> 403
curl -i -X POST $API/tasks -H "Authorization: Bearer $BOND" -H 'Content-Type: application/json' -d '{"title":"x"}'
# HTTP/1.1 403 Forbidden
# {"error":{"code":"forbidden","message":"Only admins can perform this action"}}

# 10-11. View James Bond's tasks twice
curl $API/tasks/view-my-tasks -H "Authorization: Bearer $BOND"   # cache.hit = false
curl $API/tasks/view-my-tasks -H "Authorization: Bearer $BOND"   # cache.hit = true
```

You can also run the whole flow from **Swagger UI** at <http://127.0.0.1:8080/swagger-ui>. Click *Authorize* and paste the JWT.

## Final validation response

Captured from a real run against the local stack:

```http
GET /tasks/view-my-tasks
Authorization: Bearer JAMES_BOND_TOKEN
```

First call:

```json
{
  "user": { "email": "jamesbond@example.com", "role": "staff" },
  "tasks": [
    { "id": "19918066-fdd4-4871-83c6-70e1dee4addc", "title": "Infiltrate SPECTRE meeting", "status": "todo", "priority": "high",   "assigned_to": "jamesbond@example.com" },
    { "id": "90f228bb-44a0-4ff7-8ac9-0c76aed6ec9f", "title": "Collect gadgets from Q",     "status": "todo", "priority": "medium", "assigned_to": "jamesbond@example.com" },
    { "id": "306a0da5-29c6-4352-8e4a-05241469cfc6", "title": "Brief M on findings",        "status": "todo", "priority": "low",    "assigned_to": "jamesbond@example.com" }
  ],
  "summary": { "total_assigned_tasks": 3 },
  "cache": { "hit": false }
}
```

Second call: the same body, served from Redis:

```json
{
  "user": { "email": "jamesbond@example.com", "role": "staff" },
  "tasks": [
    { "id": "19918066-fdd4-4871-83c6-70e1dee4addc", "title": "Infiltrate SPECTRE meeting", "status": "todo", "priority": "high",   "assigned_to": "jamesbond@example.com" },
    { "id": "90f228bb-44a0-4ff7-8ac9-0c76aed6ec9f", "title": "Collect gadgets from Q",     "status": "todo", "priority": "medium", "assigned_to": "jamesbond@example.com" },
    { "id": "306a0da5-29c6-4352-8e4a-05241469cfc6", "title": "Brief M on findings",        "status": "todo", "priority": "low",    "assigned_to": "jamesbond@example.com" }
  ],
  "summary": { "total_assigned_tasks": 3 },
  "cache": { "hit": true }
}
```

## Tests

```bash
docker compose up -d          # integration tests need Postgres + Redis

cd backend
cargo fmt --check
cargo clippy --all-targets -- -D warnings
DATABASE_URL=postgres://taskapp:taskapp@localhost:5432/taskapp cargo test

cd ../frontend
npm test                      # vitest
npm run build                 # type-check + production build
```

**Backend:** 29 tests.

- **Unit tests** (in the modules): Argon2 hash and verify, JWT round-trip, wrong secret, expiry, OTP generation, HMAC hashing and constant-time verify, email masking.
- **Integration tests** (`backend/tests/`): each test gets a fresh database from `#[sqlx::test]`, with migrations applied, plus its own Redis key prefix. The full Axum router is driven in-process with `tower::ServiceExt::oneshot`.

| Requirement | Test |
|---|---|
| Admin and James Bond can be created | `auth_tests::seed_creates_admin_and_james_bond` |
| Login creates a 2FA challenge, no JWT | `auth_tests::login_creates_challenge_without_returning_jwt` |
| Correct code returns a JWT | `auth_tests::correct_code_returns_jwt` |
| Incorrect code rejected | `auth_tests::incorrect_code_is_rejected` |
| Expired code rejected (5 min) | `auth_tests::expired_code_is_rejected`, `challenge_expires_after_five_minutes` |
| Reused code rejected | `auth_tests::reused_code_is_rejected` |
| Brute force limited | `auth_tests::too_many_wrong_attempts_lock_the_challenge` |
| Admin creates 5 tasks, assigns 3; Bond gets 403; Bond sees 3; cache false then true | `task_tests::full_validation_workflow` |
| Staff can't assign or list all tasks | `task_tests::staff_cannot_assign_or_list_all_tasks` |
| Assignment invalidates cache (new **and** previous assignee) | `task_tests::assignment_invalidates_cache` |
| Update invalidates cache | `task_tests::task_update_invalidates_cache` |
| Input validation / transactional assign | `task_tests::assign_validates_input`, `create_task_validates_input` |

**Frontend:** 9 vitest tests. They check that the API client attaches the JWT, skips it for public calls, maps a 403 to `ApiError`, signs out on 401, and reports network errors. They also cover the `TaskList` loading, error, empty and data states.

I also drove the full UI flow end to end in headless Chrome: login, 2FA, create 5 tasks, assign 3, the Bond view showing 3 tasks with a cache miss and then a hit, and the 403 message. The screenshots below come from that run.

## API reference

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/seed/users[?reset=true]` | dev only | Create or refresh Admin and James Bond; `reset` also wipes tasks, challenges and cache |
| POST | `/auth/login` | — | Check credentials, create a 2FA challenge, email the code. Returns `login_challenge_id` |
| GET | `/dev/email-logs/latest[?email=]` | dev only | Latest verification email, including the code |
| POST | `/auth/verify-2fa` | — | `{login_challenge_id, code}` → JWT |
| GET | `/auth/me` | any user | Current user |
| POST | `/tasks` | **admin** | Create a task |
| GET | `/tasks` | **admin** | All tasks with assignee (feeds the admin UI) |
| POST | `/tasks/assign` | **admin** | `{task_ids[], assignee_email}` |
| PATCH | `/tasks/{id}` | admin, or the assignee (status only) | Update a task |
| GET | `/tasks/view-my-tasks` | any user | Caller's tasks, with `cache.hit` |
| GET | `/users[?role=staff]` | **admin** | Users (feeds the assignee dropdown) |
| GET | `/health` | — | Liveness |
| GET | `/swagger-ui`, `/api-docs/openapi.json` | — | OpenAPI docs |

All the routes from the brief are implemented as written. I added `GET /tasks`, `PATCH /tasks/{id}`, `GET /users`, `GET /auth/me` and `GET /health`.

Errors always use one JSON envelope: `{"error": {"code": "forbidden", "message": "…"}}`. The status codes are:

| Status | When |
|---|---|
| 400 | Malformed JSON or bad enum value |
| 401 | Unauthenticated, or invalid, expired or used code. 2FA codes are `invalid_code`, `code_expired`, `code_already_used` and `invalid_challenge` |
| 403 | Wrong role |
| 404 | Not found |
| 422 | Validation failed |
| 429 | Too many 2FA attempts |
| 500 | Internal error. Details are logged, never returned |

## Design notes

### Architecture (backend)

```
routes/        thin Axum handlers + OpenAPI annotations
  └─ services/ business rules (auth flow, permissions, cache policy)
       └─ repositories/  SQL only (sqlx, any executor → transactions chosen by services)
models/        DB rows + enums       dto/   request/response contract (serde + validator + utoipa)
auth/          argon2, JWT, OTP, AuthUser/AdminUser extractors
cache.rs       Redis TaskCache        mail.rs  Mailer trait + dev ConsoleMailer
```

- **Role-based access is enforced by the type system.** A handler that takes `AdminUser` cannot run for a staff user. The extractor rejects the request with 403 before any handler code runs, and with 401 when there is no valid token.
- `build_app(state)` lives in `lib.rs`, so integration tests exercise exactly the router the binary serves.

### Two-factor authentication

- `POST /auth/login` verifies the Argon2 password hash. An unknown email still costs one Argon2 verify against a dummy hash and gets the same 401, so you can't enumerate users by timing or by message. It then creates a `login_challenges` row and "sends" an email.
- **The code is never stored in plain text.** The row stores `HMAC-SHA256(OTP_SECRET, challenge_id ‖ code)`. Binding the hash to the challenge id means a hash can't be replayed against another challenge. Verification compares in constant time.
- **Expiry:** `expires_at = now + 5 min`. Expired codes get `code_expired`.
- **Single use:** the challenge is consumed with `UPDATE … SET consumed_at = now() WHERE id = $1 AND consumed_at IS NULL AND expires_at > now()`. Only one request can win that update, so a code can't be reused, even by two simultaneous requests.
- **Brute force:** after 5 wrong attempts the challenge is locked (429). A new login also revokes any older pending challenges for that user.
- **Email delivery:** the `Mailer` trait has one implementation, `ConsoleMailer`. It logs the email to the server console and keeps it in a bounded in-memory outbox that `GET /dev/email-logs/latest` reads. The `email_logs` table records metadata only (recipient, subject, kind, challenge id, time). Adding SMTP means adding another `Mailer` implementation.
- The JWT (HS256, 60 min by default) is issued only after verification succeeds.

### Caching (Redis)

- Key `taskapp:tasks:user:{user_id}` holds the JSON of the `view-my-tasks` response, with a 300 s TTL.
- **Miss:** load from Postgres, store in Redis, return `cache.hit=false`. **Hit:** return the stored body with `cache.hit=true`.
- **Invalidation:** after the database transaction commits, `POST /tasks/assign` deletes the key for the new assignee **and** for every previous assignee of the moved tasks. `PATCH /tasks/{id}` deletes the current assignee's key. Creating a task needs no invalidation, because new tasks are unassigned.
- **Graceful degradation:** if Redis is unavailable, reads fall back to Postgres instead of failing.
- **Known limitation:** a read that misses the cache could, in theory, write its database snapshot back *after* a concurrent assignment has invalidated the key. The TTL bounds how long that stale entry can live. Per-user version counters would close the gap entirely, but that's more than this assignment needs.

### Data model

- `users(id, full_name, email UNIQUE lowercased, hashed_password, role user_role, created_at, updated_at)`
- `tasks(id, title, description, status task_status, priority task_priority, created_by_id → users, assigned_to_id → users NULL, created_at, updated_at)`
- `login_challenges(id, user_id, code_hash, expires_at, consumed_at, attempts, created_at)`
- `email_logs(id, recipient, subject, kind, challenge_id, created_at)`

Roles, statuses and priorities are native Postgres enums. `updated_at` is kept current by a trigger. `task_priority` is declared `low < medium < high`, so `ORDER BY priority DESC` returns high, medium, low.

### Frontend

- `src/api/`: the only place that talks HTTP. `client.ts` attaches `Authorization: Bearer <jwt>`, converts error envelopes into `ApiError`, and clears the session on a 401.
- `src/context/AuthContext.tsx`: holds the session (token and user) in `sessionStorage`.
- `src/components/`: reusable UI (forms, tables, alerts, spinner, empty state).
- `src/pages/`: Login (two-step), Admin, and My tasks. Each has loading, error, empty and success states.
- `<StrictMode>` is deliberately omitted. Its dev-only double mount would call `view-my-tasks` twice on page load, so the first thing you'd see is `cache.hit = true`.

## Project structure

```
.
├── docker-compose.yml          Postgres 16 + Redis 7
├── .env.example                backend env template
├── scripts/validate.{sh,ps1}   end-to-end validation
├── docs/screenshots/
├── backend/
│   ├── migrations/             SQL migrations
│   ├── src/{auth,dto,models,repositories,routes,services}/
│   ├── src/{cache,config,error,extract,mail,openapi,state,lib,main}.rs
│   └── tests/                  integration tests
└── frontend/
    └── src/{api,components,context,hooks,pages}/
```

## Screenshots

| | |
|---|---|
| Login, step 2 (2FA code) | ![2FA](docs/screenshots/01-login-2fa.png) |
| Admin creates 5 tasks and assigns 3 to James Bond | ![Admin](docs/screenshots/02-admin-create-assign.png) |
| James Bond's tasks (cache hit) and the 403 when creating a task | ![Staff](docs/screenshots/03-staff-my-tasks-403.png) |
