use std::sync::Arc;
use oracle::Connection;
use chrono::NaiveDateTime;

pub mod errors;

#[derive(Debug)]
pub struct PreppedGridData {
    pub data: Vec<Vec<FormattedData>>,
    pub conn: Connection,
}

#[derive(Debug, Clone)]
pub enum FormattedData {
    STRING(String),
    INT(i64),
    FLOAT(f64),
    DATE(NaiveDateTime),
    EMPTY,
}

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

#[derive(Debug)]
pub struct PreppedRowData {
    pub data: Vec<String>,
    pub conn: Connection,
}