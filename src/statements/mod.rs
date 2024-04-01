use oracle::Connection;
use std::sync::Arc;
use crate::statements::mutate_row::MutateRow;
use crate::types::errors::OracleSqlToolsError;
use crate::statements::mutate_grid::MutateGrid;
use crate::types::{BatchPrep, DatatypeIndexes, PreppedGridData};

use self::create_table::Create;
pub mod mutate_grid;
pub mod mutate_row;
pub mod insert_utils;
pub mod create_table;

impl PreppedGridData {
    pub fn insert(self, table_name: &str) -> Result<(), OracleSqlToolsError> {
        let table_exists = does_table_exist(&self.conn, &table_name)?;
        let data_indexes: DatatypeIndexes;
        let (data_header, data_body) = match table_exists {
            true => {
                let mut grid = self.data;
                data_indexes = grid.get_col_datatype();
                let (data_header, _) = grid.replace_header(&self.conn, &table_name)?;
                (data_header, grid)
            },
            false => {
                let mut grid = self.data;
                data_indexes = grid.get_col_datatype();
                grid.create_table(table_name, &data_indexes, &self.conn)?;
                let (data_header, _) = grid.separate_header();
                (data_header, grid)
            },
        };
        if data_header.len() != data_body[0].len() { 
            return Err(OracleSqlToolsError::InvalidHeaderLength { 
                header_length: data_header.len(), 
                body_length: data_body.len(), 
            }) 
        }
        // println!("{:?}\n===============\n{:?}", data_header, data_body);
        let conn: Arc<Connection> = Arc::new(self.conn);
        let insert_stmt = data_header.insert_stmt(table_name);

        BatchPrep {
            data_body,
            conn,
            insert_stmt,
            data_indexes,
        }.split_batch_by_threads()?;
        Ok(())
    }
}

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