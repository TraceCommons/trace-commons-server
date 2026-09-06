// Copyright (C) 2026 K&Z Partners LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Applying a migration and recording it must commit or roll back together.
//!
//! The migration runner used to `batch_execute` a migration file and then, as a
//! separate statement on the same connection, insert its row into
//! `_trace_commons_migrations`. Both statements succeed in every normal run, so
//! no test in the suite could see the difference: the gap only shows when the
//! second one does not happen. This test forces that case -- a recording table
//! that rejects the row -- and requires the migration's own DDL to be gone
//! afterwards.
//!
//! Requires PostgreSQL: set `TRACE_COMMONS_PG_TEST_DATABASE_URL` or
//! `DATABASE_URL`. CI runs no PostgreSQL, so these skip there;
//! `applying_a_migration_and_recording_it_share_one_transaction` in
//! `src/db/postgres.rs` is what gates the shape on every run.
//!
//! Everything here lives in a scratch schema of its own and is dropped again,
//! so the shared test database keeps whatever migration state it already had.

use trace_commons_server::db::postgres::apply_and_record_migration;

/// Connects and puts the session in a private scratch schema holding its own
/// `_trace_commons_migrations`. Returns `None` when no database is configured.
///
/// `search_path` is what makes this work: the runner writes to an unqualified
/// `_trace_commons_migrations`, so the scratch copy in front of the path is the
/// one it finds, and the real recording table is never touched.
async fn scratch_client(schema: &str, reject_version: i32) -> Option<tokio_postgres::Client> {
    let url = std::env::var("TRACE_COMMONS_PG_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()?;

    let (client, connection) = tokio_postgres::connect(&url, tokio_postgres::NoTls)
        .await
        .expect("connect to the configured test database");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    client
        .batch_execute(&format!(
            "DROP SCHEMA IF EXISTS {schema} CASCADE;
             CREATE SCHEMA {schema};
             SET search_path = {schema}, public;
             CREATE TABLE _trace_commons_migrations (
                 version INTEGER PRIMARY KEY,
                 name TEXT NOT NULL,
                 applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                 CONSTRAINT probe_rejects_one_version CHECK (version <> {reject_version})
             );"
        ))
        .await
        .expect("set up the scratch schema");

    Some(client)
}

async fn drop_scratch(client: &tokio_postgres::Client, schema: &str) {
    client
        .batch_execute(&format!("DROP SCHEMA IF EXISTS {schema} CASCADE;"))
        .await
        .expect("drop the scratch schema");
}

async fn probe_table_exists(client: &tokio_postgres::Client, schema: &str) -> bool {
    let exists: Option<String> = client
        .query_one(
            &format!("SELECT to_regclass('{schema}.atomicity_probe')::TEXT"),
            &[],
        )
        .await
        .expect("look the probe table up")
        .get(0);
    exists.is_some()
}

/// The failure this exists for: the record cannot be written, so the migration
/// must not stay applied. Before the transaction, `atomicity_probe` survived
/// and the next boot re-ran V999 into `relation already exists`.
#[tokio::test]
async fn a_migration_whose_recording_fails_leaves_nothing_applied() {
    let schema = "trace_migration_atomicity_rollback";
    let Some(client) = scratch_client(schema, 999).await else {
        eprintln!("skipping: TRACE_COMMONS_PG_TEST_DATABASE_URL or DATABASE_URL not configured");
        return;
    };
    let mut client = client;

    let result = apply_and_record_migration(
        &mut client,
        999,
        "atomicity_probe",
        "CREATE TABLE atomicity_probe (id INTEGER PRIMARY KEY);",
    )
    .await;

    assert!(
        result.is_err(),
        "the CHECK constraint must reject version 999, or this test proves nothing"
    );
    assert!(
        !probe_table_exists(&client, schema).await,
        "the migration's table outlived the failed recording: the apply and the \
         record are not in one transaction, and a crash between them leaves an \
         applied-but-unrecorded migration that fails every later boot"
    );

    let recorded: i64 = client
        .query_one("SELECT COUNT(*) FROM _trace_commons_migrations", &[])
        .await
        .expect("count recordings")
        .get(0);
    assert_eq!(
        recorded, 0,
        "nothing may be recorded when the insert failed"
    );

    drop_scratch(&client, schema).await;
}

/// The rollback is not bought by refusing to apply anything: a migration whose
/// recording succeeds must leave both its DDL and its row behind.
#[tokio::test]
async fn a_migration_that_records_cleanly_keeps_both_halves() {
    let schema = "trace_migration_atomicity_commit";
    let Some(client) = scratch_client(schema, -1).await else {
        eprintln!("skipping: TRACE_COMMONS_PG_TEST_DATABASE_URL or DATABASE_URL not configured");
        return;
    };
    let mut client = client;

    apply_and_record_migration(
        &mut client,
        999,
        "atomicity_probe",
        "CREATE TABLE atomicity_probe (id INTEGER PRIMARY KEY);",
    )
    .await
    .expect("a migration with a writable recording must apply");

    assert!(
        probe_table_exists(&client, schema).await,
        "the migration's own DDL must be durable after the commit"
    );

    let row = client
        .query_one("SELECT version, name FROM _trace_commons_migrations", &[])
        .await
        .expect("read the recording");
    let version: i32 = row.get(0);
    let name: String = row.get(1);
    assert_eq!(
        (version, name.as_str()),
        (999, "atomicity_probe"),
        "the recorded (version, name) must be exactly what was applied"
    );

    drop_scratch(&client, schema).await;
}
