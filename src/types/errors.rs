use thiserror::Error;

#[derive(Error, Debug)]
pub enum OracleSqlToolsError {
    #[error(transparent)]
    OracleError(#[from] oracle::Error),
    
    #[error("Header length, {header_length}, does not match body length, {body_length}")]
    InvalidHeaderLength {header_length: usize, body_length: usize},

    #[error("Input grid length is less than or equal to 1, which implies the grid is empty or only contains a header.")]
    NoData,

    #[error("{:?}\n Cell Value:{:?} X_Index:{:?}, Y_INDEX:{:?}", error_message, cell_value, x_index, y_index)]
    CellPropertyError {
        error_message: oracle::Error,
        cell_value: String,
        x_index: usize,
        y_index: usize,
    },

    #[error(transparent)]
    DateCantConvertToString(#[from] core::convert::Infallible)
}