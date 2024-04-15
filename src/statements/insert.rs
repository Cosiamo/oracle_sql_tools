use std::sync::Arc;
use oracle::Connection;

use crate::types::{errors::OracleSqlToolsError, BatchPrep, DatatypeIndexes};
use super::{create_table::CreateFromInsert, mutate_grid::MutateGrid, mutate_row::MutateRow, utils::does_table_exist, PreppedGridData};

impl PreppedGridData {
    // Inserts the input data into a table
    //
    // Splits the data by the number of CPU threads in the host machine. Each thread creates it's own [`oracle::Batch`] which helps the upload speed for large datasets.
    pub fn insert(self, table_name: &str) -> Result<Arc<Connection>, OracleSqlToolsError> {
        stage_insert_data(self, table_name)
            .unwrap()
            .split_batch_by_threads()
    }

    // Inserts the input data into a table using only a single thread
    //
    // Useful if you have a procedure or trigger in your database that groups data in a table or view via insert
    pub fn insert_single_thread(self, table_name: &str) -> Result<Arc<Connection>, OracleSqlToolsError> {
        stage_insert_data(self, table_name)
            .unwrap()
            .single_thread_batch()
    }
}

fn stage_insert_data(mut grid_data: PreppedGridData, table_name: &str) -> Result<BatchPrep, OracleSqlToolsError> {
    let table_exists = does_table_exist(&grid_data.conn, &table_name)?;
    // get's the 'dominate' datatype from each column
    // weighted in order: VARCHAR2, FLOAT, INT, DATE
    let data_indexes: DatatypeIndexes;
    let (data_header, data_body) = match table_exists {
        // if the user input table exists, it replaces the header with the column names from the table
        true => {
            data_indexes = grid_data.data.get_varchar_ind();
            let (data_header, _) = grid_data.data.replace_header(&grid_data.conn, &table_name)?;
            (data_header, grid_data.data)
        },
        // if user input table does not exist, it creates a new table
        false => {
            data_indexes = grid_data.data.get_col_datatype();
            grid_data.data.create_table(table_name, &data_indexes, &grid_data.conn)?;
            let (data_header, _) = grid_data.data.separate_header();
            (data_header, grid_data.data)
        },
    };
    if data_header.len() != data_body[0].len() { 
        return Err(OracleSqlToolsError::InvalidHeaderLength { 
            header_length: data_header.len(), 
            body_length: data_body.len(), 
        }) 
    }

    Ok(BatchPrep {
        data: data_body,
        conn: grid_data.conn,
        insert_stmt: data_header.insert_stmt(table_name),
        data_indexes,
    })
}