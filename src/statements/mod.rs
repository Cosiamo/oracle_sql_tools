use oracle::Connection;

use crate::format_data::FormattedData;

pub mod mutate_grid;
pub mod mutate_row;
pub mod insert_utils;
pub mod create_table;
pub mod insert;
pub mod utils;
pub mod select;

#[derive(Debug)]
pub struct PreppedGridData {
    pub data: Vec<Vec<FormattedData>>,
    pub conn: Connection,
}

#[derive(Debug)]
pub struct PreppedRowData {
    pub data: Vec<String>,
    pub conn: Connection,
}
