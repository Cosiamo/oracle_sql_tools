use crate::types::{errors::OracleSqlToolsError, BatchPrep, DatatypeIndexes, PreppedGridData};
use super::{create_table::Create, mutate_grid::MutateGrid, mutate_row::MutateRow, utils::does_table_exist};

impl PreppedGridData {
    // multi-threaded approach by default
    // splits the data by the number of CPU threads in the host machine
    // in each thread it creates it's own Batch
    pub fn insert(self, table_name: &str) -> Result<(), OracleSqlToolsError> {
        let batch_prep = stage_insert_data(self, table_name)?;
        batch_prep.split_batch_by_threads()?;
        Ok(())
    }

    // only uses a single core to insert the data
    pub fn insert_single_thread(self, table_name: &str) -> Result<(), OracleSqlToolsError> {
        let batch_prep = stage_insert_data(self, table_name)?;
        batch_prep.single_thread_batch()?;
        Ok(())
    }
}

fn stage_insert_data(grid_data: PreppedGridData, table_name: &str) -> Result<BatchPrep, OracleSqlToolsError> {
    let table_exists = does_table_exist(&grid_data.conn, &table_name)?;
    // get's the 'dominate' datatype from each column
    // weighted in order: VARCHAR2, FLOAT, INT, DATE
    let data_indexes: DatatypeIndexes = grid_data.data.get_col_datatype();
    let (data_header, data_body) = match table_exists {
        // if the user input table exists, it replaces the header with the column names from the table
        true => {
            let mut grid = grid_data.data;
            let (data_header, _) = grid.replace_header(&grid_data.conn, &table_name)?;
            (data_header, grid)
        },
        // if user input table does not exist, it creates a new table
        false => {
            let mut grid = grid_data.data;
            grid.create_table(table_name, &data_indexes, &grid_data.conn)?;
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
    // let conn: Arc<Connection> = Arc::new(self.conn);
    let insert_stmt: String = data_header.insert_stmt(table_name);

    Ok(BatchPrep {
        data: data_body,
        conn: grid_data.conn,
        insert_stmt,
        data_indexes,
    })
}