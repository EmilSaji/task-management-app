# AI Usage

## Tools used

- **Claude Code (Anthropic, Claude Opus model)** in the terminal, used for:
  - planning the architecture (stack choice, module layout, 2FA and cache design)
  - generating the backend, frontend, migrations, tests, validation scripts, CI workflow and documentation
  - running the build, clippy, the test suites and the end-to-end validation against the local Docker stack
  - driving the frontend in headless Chrome to capture the screenshots

## How the work was steered

I made the stack decisions before any code was written: PostgreSQL + Redis through Docker Compose, Axum, and Vite + React + TypeScript. The plan was reviewed and approved before implementation started.

These issues were found and fixed during development, not just generated:

- **Token race on the first request after login.** React runs child effects before parent effects, so the first fetch after login would have been sent without the JWT. The API client is now configured synchronously, through a ref, in `AuthContext`.
- **StrictMode double fetch.** In development, StrictMode would call `view-my-tasks` twice on mount, so the UI would show `cache.hit = true` on first load. StrictMode was removed, and the reason is documented in `main.tsx`.
- **Subshell bug in `validate.sh`.** The HTTP status variable was being set inside `$(...)` subshells, where the caller couldn't see it. The status is now written to a file.
- **Clippy MSRV lint.** `Option::is_none_or` requires Rust 1.82, so the declared `rust-version` was raised to 1.85.

## What I reviewed / changed manually

-

## What I can explain

Everything in the repo, in particular:

- why the 2FA code is stored as an HMAC bound to the challenge id, and how single-use is guaranteed atomically
- why `AdminUser` / `AuthUser` extractors enforce RBAC before handler code runs
- the cache key design, why invalidation runs after commit, and the documented stale-write race
- how integration tests get an isolated database (`#[sqlx::test]`) and Redis namespace per test
