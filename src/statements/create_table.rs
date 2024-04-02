use std::collections::HashMap;
use oracle::Connection;

use crate::{types::{errors::OracleSqlToolsError, DatatypeIndexes, FormattedData}, utils::remove_invalid_chars};

use super::mutate_row::MutateRow;

pub trait Create {
    fn create_table(
        &self, table_name: &str, col_indexes: &DatatypeIndexes, conn: &Connection
    ) -> Result<(), OracleSqlToolsError>;
}

macro_rules! compare_data_length {
    ($varchar_col_size:ident, $val:ident, $x:ident) => {
        match $varchar_col_size.get(&$x) {
            Some(prev_val) => {
                if $val.to_string().len() > *prev_val { $varchar_col_size.insert($x, $val.to_string().len()) }
                else { continue }
            },
            None => $varchar_col_size.insert($x, $val.to_string().len()),
        }
    };
}

impl Create for Vec<Vec<FormattedData>> {
    fn create_table(
        &self, table_name: &str, data_type_indexes: &DatatypeIndexes, conn: &Connection
    ) -> Result<(), OracleSqlToolsError> {
        if self.len() <= 1 { return Err(OracleSqlToolsError::NoData); }

        let mut varchar_col_size: HashMap<usize, usize> = HashMap::new();
        for x in 0..self[0].len() {
            if !data_type_indexes.is_varchar.contains(&x) { continue; };
            for y in 1..self.len() {
                match &self[y][x] {
                    FormattedData::STRING(val) => compare_data_length!(varchar_col_size, val, x),
                    FormattedData::INT(val) => compare_data_length!(varchar_col_size, val, x),
                    FormattedData::FLOAT(val) => compare_data_length!(varchar_col_size, val, x),
                    FormattedData::DATE(val) => compare_data_length!(varchar_col_size, val, x),
                    FormattedData::EMPTY => { let val = 0 as usize; compare_data_length!(varchar_col_size, val, x) },
                };
            }
        }

        let mut sql_data_types = Vec::new();
        for x in 0..self[0].len() {
            if data_type_indexes.is_varchar.contains(&x) {
                match varchar_col_size.get(&x) {
                    Some(val) => sql_data_types.push(format!("VARCHAR2({})", val)),
                    None => continue,
                }
            } else if data_type_indexes.is_int.contains(&x) { sql_data_types.push(format!("NUMBER")) }
            else if data_type_indexes.is_float.contains(&x) { sql_data_types.push(format!("FLOAT")) }
            else if data_type_indexes.is_date.contains(&x) { sql_data_types.push(format!("DATE")) }
            else { sql_data_types.push(format!("")) }
        }

        let mut col_names = Vec::new();
        for (i, col_header) in self[0].to_string().iter().enumerate() {
            col_names.push(format!("{} {}", remove_invalid_chars(col_header), sql_data_types[i]))
        }
        let create_table_stmt = format!("CREATE TABLE {} ({})", table_name, col_names.join(", "));
        conn.execute(&create_table_stmt, &[])?;
        Ok(conn.commit()?)
    }
}