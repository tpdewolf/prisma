use failure::{Error, Fail};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod mysql;
pub mod postgres;
pub mod sqlite;

/// Introspection errors.
#[derive(Debug, Fail)]
pub enum IntrospectionError {
    /// An unknown error occurred.
    #[fail(display = "unknown")]
    UnknownError,
}

/// The result type.
pub type Result<T> = core::result::Result<T, Error>;

/// Connection abstraction for the introspection connectors.
pub trait IntrospectionConnection {
    fn query_raw(&self, sql: &str, schema: &str) -> prisma_query::Result<prisma_query::connector::ResultSet>;
}

/// A database introspection connector.
pub trait IntrospectionConnector {
    /// List the database's schemas.
    fn list_schemas(&self) -> Result<Vec<String>>;
    /// Introspect a database schema.
    fn introspect(&self, schema: &str) -> Result<DatabaseSchema>;
}

/// The result of introspecting a database schema.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DatabaseSchema {
    /// The schema's tables.
    pub tables: Vec<Table>,
    /// The schema's enums.
    pub enums: Vec<Enum>,
    /// The schema's sequences, unique to Postgres.
    pub sequences: Vec<Sequence>,
}

impl DatabaseSchema {
    /// Get a table.
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|x| x.name == name)
    }

    /// Get an enum.
    pub fn get_enum(&self, name: &str) -> Option<&Enum> {
        self.enums.iter().find(|x| x.name == name)
    }

    pub fn get_sequence(&self, name: &str) -> Option<&Sequence> {
        self.sequences.iter().find(|x| x.name == name)
    }
}

/// A table found in a schema.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// The table's name.
    pub name: String,
    /// The table's columns.
    pub columns: Vec<Column>,
    /// The table's indices.
    pub indices: Vec<Index>,
    /// The table's primary key, if there is one.
    pub primary_key: Option<PrimaryKey>,
    /// The table's foreign keys.
    pub foreign_keys: Vec<ForeignKey>,
}

/// An index of a table.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    /// Index name.
    pub name: String,
    /// Index columns.
    pub columns: Vec<String>,
    /// Is index unique?
    pub unique: bool,
}

/// The primary key of a table.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryKey {
    /// Columns.
    pub columns: Vec<String>,
}

/// A column of a table.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Column {
    /// Column name.
    pub name: String,
    /// Column type.
    pub tpe: ColumnType,
    /// Column arity.
    pub arity: ColumnArity,
    /// Column default.
    // Does this field need to be richer? E.g. to easier detect the usages of sequences here
    pub default: Option<String>,
    /// Column auto increment setting, MySQL/SQLite only.
    pub auto_increment: bool,
}

/// The type of a column.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct ColumnType {
    /// The raw SQL type.
    pub raw: String,
    /// The family of the raw type.
    pub family: ColumnTypeFamily,
}

/// Enumeration of column type families.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
// TODO: this name feels weird.
pub enum ColumnTypeFamily {
    /// Integer types.
    Int,
    /// Floating point types.
    Float,
    /// Boolean types.
    Boolean,
    /// String types.
    String,
    /// DateTime types.
    DateTime,
    /// Binary types.
    Binary,
    /// JSON types.
    Json,
    /// UUID types.
    Uuid,
    /// Geometric types.
    Geometric,
    /// Log sequence number types.
    LogSequenceNumber,
    /// Text search types.
    TextSearch,
    /// Transaction ID types.
    TransactionId,
}

/// A column's arity.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum ColumnArity {
    /// Required column.
    Required,
    /// Nullable column.
    Nullable,
    /// List type column.
    List,
}

/// Foreign key action types (for ON DELETE|ON UPDATE) constraints.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum ForeignKeyAction {
    /// Produce an error indicating that the deletion or update would create a foreign key
    /// constraint violation. If the constraint is deferred, this error will be produced at
    /// constraint check time if there still exist any referencing rows. This is the default action.
    NoAction,
    /// Produce an error indicating that the deletion or update would create a foreign key
    /// constraint violation. This is the same as NO ACTION except that the check is not deferrable.
    Restrict,
    /// Delete any rows referencing the deleted row, or update the values of the referencing
    /// column(s) to the new values of the referenced columns, respectively.
    Cascade,
    /// Set the referencing column(s) to null.
    SetNull,
    /// Set the referencing column(s) to their default values. (There must be a row in the
    /// referenced table matching the default values, if they are not null, or the operation
    /// will fail).
    SetDefault,
}

/// A foreign key.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    /// Column names.
    pub columns: Vec<String>,
    /// Referenced table.
    pub referenced_table: String,
    /// Referenced columns.
    pub referenced_columns: Vec<String>,
    pub on_delete_action: ForeignKeyAction,
}

/// A SQL enum.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Enum {
    /// Enum name.
    pub name: String,
    /// Possible enum values.
    pub values: HashSet<String>,
}

/// A SQL sequence.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Sequence {
    /// Sequence name.
    pub name: String,
    /// Sequence initial value.
    pub initial_value: u32,
    /// Sequence allocation size.
    pub allocation_size: u32,
}
