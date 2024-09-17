use oracle::Connection;

use crate::{format_data::FormattedData, types::DatatypeIndexes};

pub mod mutate_grid;
pub mod mutate_row;
pub mod create_table;
pub mod insert;
pub mod utils;
pub mod select;

#[derive(Debug)]
pub struct PreppedGridData {
    pub data: Vec<Vec<FormattedData>>,
    pub conn: Connection,
    pub data_indexes: DatatypeIndexes,
}

#[derive(Debug)]
pub struct PreppedRowData {
    pub data: Vec<String>,
    pub conn: Connection,
    pub query: Option<String>,
    pub header: Option<Vec<String>>,
    pub filters: Option<Vec<String>>,
}