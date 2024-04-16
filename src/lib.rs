#![doc = include_str!("../README.md")]

use oracle::Connection;

use statements::{PreppedGridData, PreppedRowData};
use format_data::{FormatData, FormattedData};
use types::DatatypeIndexes;

pub mod statements;
pub mod types;
pub mod utils;
pub mod format_data;

/// A trait to prepare either a vector or a 2-dimensional vector for a SQL query
///
/// The trait either returns [`statements::PreppedRowData`] or [`statements::PreppedGridData`] respectively
///
/// Using a vector to select specific columns from a table:
///
/// ```no_run
/// let conn: oracle::Connection = match Connection::connect("<USERNAME>", "<PASSWORD>", "<IP ADDRESS>")?; 
///
/// let col_names: Vec<&str> = vec!["Employee ID", "Name", "Job Title", "Department", "Business Unit"];
///
/// let table_data: Vec<Vec<Option<String>>> = col_names.prep_data(conn).select("MY_TABLE")?;
/// ```
///
/// Using a 2-dimensional vector to insert data:
///
/// ```no_run
/// let conn: oracle::Connection = match Connection::connect("<USERNAME>", "<PASSWORD>", "<IP ADDRESS>")?; 
///
/// let data: Vec<Vec<String>> = vec![
///     vec!["ColA".to_string(), "ColB".to_string(), "ColC".to_string()],
///     vec!["A1".to_string(), "B1".to_string(), "C1".to_string()],
///     vec!["A2".to_string(), "B2".to_string(), "C2".to_string()],
///     vec!["A3".to_string(), "B3".to_string(), "C3".to_string()],
/// ];
/// 
/// data.prep_data(conn).insert("MY_TABLE")?;
/// Ok(())
/// ```
pub trait PrepData<T: FormatData> {
    type Prep;

    fn prep_data(self, connection: Connection) -> Self::Prep;
}

impl<T: FormatData> PrepData<T> for Vec<Vec<T>> {
    type Prep = PreppedGridData;

    fn prep_data(self, connection: Connection) -> Self::Prep  {
        // get's the 'dominate' datatype from each column
        // weighted in order: VARCHAR2, FLOAT, INT, DATE
        let mut is_varchar: Vec<usize> = Vec::new();
        let mut is_float: Vec<usize> = Vec::new();
        let mut is_int: Vec<usize> = Vec::new();
        let mut is_date: Vec<usize> = Vec::new();

        let mut data = Vec::new();
        for row in self {
            let mut inner_vec = Vec::new();
            let mut x_index: usize = 0 as usize;
            for cell in row {
                let formatted_cell = cell.fmt_data();
                match &formatted_cell {
                    FormattedData::STRING(_) => is_varchar.push(x_index),
                    FormattedData::INT(_) => is_int.push(x_index),
                    FormattedData::FLOAT(_) => is_float.push(x_index),
                    FormattedData::DATE(_) => is_date.push(x_index),
                    FormattedData::EMPTY => { 
                        inner_vec.push(formatted_cell); 
                        x_index += 1 as usize; 
                        continue;
                    },
                }
                inner_vec.push(formatted_cell);
                x_index += 1 as usize;
            }
            data.push(inner_vec)
        }

        let data_indexes = DatatypeIndexes {
            is_varchar,
            is_float,
            is_int,
            is_date,
        }.find_uniques();

        Self::Prep {
            data,
            conn: connection,
            data_indexes
        }
    }
}

impl<T: FormatData> PrepData<T> for Vec<T> {
    type Prep = PreppedRowData;

    fn prep_data(self, connection: Connection) -> Self::Prep {
        let mut data = Vec::new();
        for val in self { data.push(val.fmt_data().to_string()) }
        Self::Prep {
            data,
            conn: connection,
        }
    }
}