// Integration tests for database CRUD against a temp SQLite file.
//
// The production code routes all queries through a module-level LazyLock pool,
// so these tests instead stand up a fresh SQLite file, apply the schema
// directly (mirroring the embedded migrations), and verify the same SQL
// patterns that queries.rs uses.

use std::path::PathBuf;

use diesel::prelude::*;
use diesel::sql_query;

const SCHEMA_SQL: &str = "\
CREATE TABLE library (\
    uuid TEXT NOT NULL PRIMARY KEY,\
    is_local BOOLEAN NOT NULL DEFAULT FALSE,\
    data TEXT,\
    favicon BLOB\
);";

#[derive(QueryableByName, Debug)]
struct LibraryRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    uuid: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_local: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    data: Option<String>,
}

/// Stand up a fresh SQLite file at our own path, apply the schema, and
/// hand back the live connection plus a cleanup guard.
fn setup_db() -> (SqliteConnection, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut path = PathBuf::from(dir.path());
    path.push("library.db");

    let mut conn = SqliteConnection::establish(path.to_str().expect("utf8"))
        .expect("Failed to connect to sqlite");

    sql_query(SCHEMA_SQL)
        .execute(&mut conn)
        .expect("schema applied");

    (conn, dir)
}

fn insert_row(conn: &mut SqliteConnection, uuid: &str, is_local: bool, data: Option<&str>) {
    sql_query("INSERT INTO library (uuid, is_local, data) VALUES (?, ?, ?)")
        .bind::<diesel::sql_types::Text, _>(uuid)
        .bind::<diesel::sql_types::Bool, _>(is_local)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data)
        .execute(conn)
        .expect("insert succeeded");
}

fn load_stations(conn: &mut SqliteConnection) -> Vec<LibraryRow> {
    sql_query("SELECT uuid, is_local, data FROM library ORDER BY uuid")
        .load::<LibraryRow>(conn)
        .expect("load succeeded")
}

#[test]
fn insert_and_select_roundtrip() {
    let (mut conn, _guard) = setup_db();
    insert_row(&mut conn, "station-1", false, Some("{\"name\":\"Test\"}"));

    let stations = load_stations(&mut conn);
    assert_eq!(stations.len(), 1);
    assert_eq!(stations[0].uuid, "station-1");
    assert_eq!(stations[0].is_local, false);
    assert_eq!(stations[0].data.as_deref(), Some("{\"name\":\"Test\"}"));
}

#[test]
fn insert_duplicate_key_rejected() {
    use diesel::result::DatabaseErrorKind::UniqueViolation;
    use diesel::result::Error::DatabaseError;

    let (mut conn, _guard) = setup_db();
    insert_row(&mut conn, "dup", false, None);

    let result = sql_query("INSERT INTO library (uuid, is_local, data) VALUES (?, ?, ?)")
        .bind::<diesel::sql_types::Text, _>("dup")
        .bind::<diesel::sql_types::Bool, _>(false)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
        .execute(&mut conn);

    match result {
        Err(DatabaseError(UniqueViolation, _)) => {}
        other => panic!("expected UniqueViolation, got {:?}", other),
    }
}

#[test]
fn update_station() {
    let (mut conn, _guard) = setup_db();
    insert_row(&mut conn, "st-2", false, Some("v1"));

    let rows_affected = sql_query("UPDATE library SET data = ? WHERE uuid = ?")
        .bind::<diesel::sql_types::Text, _>("v2")
        .bind::<diesel::sql_types::Text, _>("st-2")
        .execute(&mut conn)
        .expect("update ok");
    assert_eq!(rows_affected, 1);

    let stations = load_stations(&mut conn);
    assert_eq!(stations[0].data.as_deref(), Some("v2"));
}

#[test]
fn delete_station() {
    let (mut conn, _guard) = setup_db();
    insert_row(&mut conn, "del-1", true, None);
    insert_row(&mut conn, "del-2", false, None);

    let deleted = sql_query("DELETE FROM library WHERE uuid = ?")
        .bind::<diesel::sql_types::Text, _>("del-1")
        .execute(&mut conn)
        .expect("delete ok");
    assert_eq!(deleted, 1);

    let stations = load_stations(&mut conn);
    assert_eq!(stations.len(), 1);
    assert_eq!(stations[0].uuid, "del-2");
}

#[test]
fn delete_missing_noop() {
    let (mut conn, _guard) = setup_db();
    let deleted = sql_query("DELETE FROM library WHERE uuid = ?")
        .bind::<diesel::sql_types::Text, _>("does-not-exist")
        .execute(&mut conn)
        .expect("delete ok");
    assert_eq!(deleted, 0);
}
