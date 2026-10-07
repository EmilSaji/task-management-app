//! Data access. Each function takes any Postgres executor (`&PgPool` or a
//! transaction), so services decide transaction boundaries.

pub mod challenges;
pub mod email_logs;
pub mod tasks;
pub mod users;
