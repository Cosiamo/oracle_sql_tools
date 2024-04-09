use std::sync::Arc;
use oracle::Connection;

use crate::format_data::FormattedData;

pub mod errors;

#[derive(Debug)]
pub struct DatatypeIndexes {
    pub is_varchar: Vec<usize>,
    pub is_float: Vec<usize>,
    pub is_int: Vec<usize>,
    pub is_date: Vec<usize>,
}

#[derive(Debug)]
pub struct BatchPrep {
    pub data: Vec<Vec<FormattedData>>,
    pub conn: Connection,
    pub insert_stmt: String,
    pub data_indexes: DatatypeIndexes,
}

#[derive(Debug)]
pub struct GridProperties {
    pub data: Arc<Vec<Vec<FormattedData>>>,
    pub num: usize,
    pub varchar_ind: Arc<Vec<usize>>,
}

#[derive(Debug)]
pub struct CellProperties<'a> {
    pub cell: &'a FormattedData,
    pub varchar_ind: &'a Arc<Vec<usize>>,
    pub x_ind: usize,
    pub y_ind: usize,
}