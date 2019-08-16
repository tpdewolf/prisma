#![allow(non_snake_case)]
#![allow(unused)]

use barrel::{types, Migration};
use database_introspection::*;
use log::{debug, LevelFilter};
use pretty_assertions::assert_eq;
use prisma_query::connector::{Queryable, Sqlite as SqliteDatabaseClient};
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{thread, time};

const SCHEMA: &str = "DatabaseInspectorTest";

static IS_SETUP: AtomicBool = AtomicBool::new(false);

fn setup() {
    let is_setup = IS_SETUP.load(Ordering::Relaxed);
    if is_setup {
        return;
    }

    let log_level = match std::env::var("RUST_LOG")
        .unwrap_or("warn".to_string())
        .to_lowercase()
        .as_ref()
    {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Warn,
    };
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!("[{}][{}] {}", record.target(), record.level(), message))
        })
        .level(log_level)
        .chain(std::io::stdout())
        .apply()
        .expect("fern configuration");

    IS_SETUP.store(true, Ordering::Relaxed);
}

#[test]
fn database_schema_is_serializable() {
    setup();

    let mut enum_values = HashSet::new();
    enum_values.insert("option1".to_string());
    enum_values.insert("option2".to_string());
    let schema = DatabaseSchema {
        tables: vec![
            Table {
                name: "table1".to_string(),
                columns: vec![
                    Column {
                        name: "column1".to_string(),
                        tpe: ColumnType {
                            raw: "integer".to_string(),
                            family: ColumnTypeFamily::Int,
                        },
                        arity: ColumnArity::Required,
                        default: None,
                        auto_increment: true,
                    },
                    Column {
                        name: "column2".to_string(),
                        tpe: ColumnType {
                            raw: "varchar(255)".to_string(),
                            family: ColumnTypeFamily::String,
                        },
                        arity: ColumnArity::Nullable,
                        default: Some("default value".to_string()),
                        auto_increment: false,
                    },
                    Column {
                        name: "column3".to_string(),
                        tpe: ColumnType {
                            raw: "integer".to_string(),
                            family: ColumnTypeFamily::Int,
                        },
                        arity: ColumnArity::Required,
                        default: None,
                        auto_increment: false,
                    },
                ],
                indices: vec![Index {
                    name: "column2".to_string(),
                    columns: vec!["column2".to_string()],
                    unique: false,
                }],
                primary_key: Some(PrimaryKey {
                    columns: vec!["column1".to_string()],
                }),
                foreign_keys: vec![ForeignKey {
                    columns: vec!["column3".to_string()],
                    referenced_table: "table2".to_string(),
                    referenced_columns: vec!["id".to_string()],
                    on_delete_action: ForeignKeyAction::NoAction,
                }],
            },
            Table {
                name: "table2".to_string(),
                columns: vec![Column {
                    name: "id".to_string(),
                    tpe: ColumnType {
                        raw: "integer".to_string(),
                        family: ColumnTypeFamily::Int,
                    },
                    arity: ColumnArity::Required,
                    default: None,
                    auto_increment: true,
                }],
                indices: vec![],
                primary_key: Some(PrimaryKey {
                    columns: vec!["id".to_string()],
                }),
                foreign_keys: vec![],
            },
        ],
        enums: vec![Enum {
            name: "enum1".to_string(),
            values: enum_values,
        }],
        sequences: vec![Sequence {
            name: "sequence1".to_string(),
            initial_value: 1,
            allocation_size: 32,
        }],
    };
    let ref_schema_json = include_str!("./resources/schema.json");
    let ref_schema: DatabaseSchema = serde_json::from_str(ref_schema_json).expect("deserialize reference schema");

    let schema_json = serde_json::to_string(&schema).expect("serialize schema to JSON");
    let schema_deser: DatabaseSchema = serde_json::from_str(&schema_json).expect("deserialize schema");

    // Verify that deserialized schema is equivalent
    assert_eq!(schema_deser, schema);
    // Verify that schema deserialized from reference JSON is equivalent
    assert_eq!(ref_schema, schema);
}
