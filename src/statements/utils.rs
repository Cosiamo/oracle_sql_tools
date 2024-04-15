use oracle::Connection;

use crate::types::errors::OracleSqlToolsError;

/// Checks if a table exists
pub fn does_table_exist(conn: &Connection, table_name: &str) -> Result<bool, OracleSqlToolsError> {
    let mut existing_tables = conn
        .statement("SELECT table_name FROM user_tables")
        .build()?;
    for row_result in existing_tables.query_as::<String>(&[])? {
        let name = row_result?;
        match name.eq_ignore_ascii_case(&table_name) {
            true => return Ok(true),
            false => continue,
        }
    }
    Ok(false)
}