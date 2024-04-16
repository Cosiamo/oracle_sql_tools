use std::{fmt::Display, sync::Arc, thread::{self, JoinHandle}};
use indicatif::ProgressBar;
use oracle::{Batch, Connection};

use crate::{format_data::FormattedData, types::{errors::OracleSqlToolsError, BatchPrep, CellProperties, DatatypeIndexes, GridProperties}, utils::remove_invalid_chars};
use super::mutate_grid::MutateGrid;

impl BatchPrep {
    pub(crate) fn split_batch_by_threads(mut self) -> Result<Arc<Connection>, OracleSqlToolsError> {
        // wrapping these variables in an Arc because they're going to be passed to multiple threads
        let conn: Arc<Connection> = Arc::new(self.conn);
        let insert_stmt: Arc<String> = Arc::new(self.insert_stmt);
        let datatype_indexes: Arc<DatatypeIndexes> = Arc::new(self.data_indexes);

        // divides the length of the data by the number of threads on the host CPU
        let len = self.data.len();
        let nthreads = num_cpus::get();
        let num = (len / nthreads + if len % nthreads == 0 { 0 } else { 1 }) as f32;

        // initialize the progress bar
        let pb = ProgressBar::new(len as u64);
        let progress_bar = Arc::new(pb);

        // captures the spawned threads into a vector
        let mut handles: Vec<JoinHandle<Result<(), OracleSqlToolsError>>> = Vec::new();
        // iterates as many times as there are threads
        for n in 0..nthreads {
            // each thread needs to have it's own clone of the data
            let conn = Arc::clone(&conn);
            let insert = Arc::clone(&insert_stmt);
            let datatype_indexes = Arc::clone(&datatype_indexes);
            let progress_bar = Arc::clone(&progress_bar);
            let arc_data: Arc<Vec<Vec<FormattedData>>>;
            if n + 1 < nthreads {
                // splits up the 2d vector to have an even amount per thread
                let d = self.data.divide(num);
                // new ARC per each split vector
                arc_data = Arc::new(d);
            // collecting the remaining data
            } else { arc_data = Arc::new(self.data.to_owned()); }
            handles.push(thread::spawn(move || {
                // creates a unique Batch per thread
                let mut batch: Batch<'_> = conn
                    .batch(&insert.as_str(), arc_data.len())
                    .build()?;
                // each thread iterates over their slice of the data
                GridProperties {
                    data: arc_data,
                    num: (num.ceil() as usize - 1) * n,
                    datatype_indexes,
                }.get_cell_props(&mut batch, progress_bar)
            }));
        }
        // executes all threads
        for handle in handles {
            handle.join().unwrap()?;
        }
        Ok(conn)
    }

    pub(crate) fn single_thread_batch(self) -> Result<Arc<Connection>, OracleSqlToolsError> {
        let body_len = &self.data.len();
        // initialize the progress bar
        let pb = ProgressBar::new(*body_len as u64);
        let progress_bar = Arc::new(pb);
        let conn: Arc<Connection> = Arc::new(self.conn);
        let conn_clone = Arc::clone(&conn);
        let mut batch: Batch<'_> = conn_clone
            .batch(&self.insert_stmt.as_str(), body_len.to_owned())
            .build()?;
        // CellProperties expects an Arc<DatatypeIndexes>
        let datatype_indexes = Arc::new(self.data_indexes);
        GridProperties {
            data: self.data.into(),
            num: 0 as usize,
            datatype_indexes,
        }.get_cell_props(&mut batch, progress_bar)?;
        Ok(conn)
    }
}

impl GridProperties {
    fn get_cell_props(self, batch: &mut Batch<'_>, progress_bar: Arc<ProgressBar>) -> Result<(), OracleSqlToolsError> {
        self.data.iter().enumerate().try_for_each(|(y, row)| 
        -> Result<(), OracleSqlToolsError> {
            row.iter().enumerate().try_for_each(|(x, cell)| 
            -> Result<(), OracleSqlToolsError> {
                CellProperties {
                    cell,
                    datatype_indexes: &self.datatype_indexes,
                    x_ind: x,
                    y_ind: (self.num + y),
                }.bind_cell_to_batch(batch)
            })?;
            batch.append_row(&[])?;
            progress_bar.inc(1 as u64);
            Ok(())
        })?;

        batch.execute()?;
        Ok(())
    }
}

macro_rules! empty_batch_set {
    ($cell_props:ident, $data_type:ty, $batch:ident) => {
        match $batch.set($cell_props.x_ind + 1, &None::<$data_type>) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(OracleSqlToolsError::CellPropertyError { 
                error_message: e, 
                cell_value: "NULL".to_string(),
                x_index: $cell_props.x_ind, 
                y_index: $cell_props.y_ind 
            }),
        }
    };
}

impl<'props> CellProperties<'props> {
    fn bind_cell_to_batch(self, batch: &mut Batch<'_>) -> Result<(), OracleSqlToolsError> {
        match &self.cell {
            FormattedData::STRING(val) => batch_set(self, batch, val.to_string()),
            FormattedData::INT(val) => match self.datatype_indexes.is_varchar.contains(&self.x_ind) {
                true => batch_set(self, batch, val.to_string()),
                false => batch_set(self, batch, *val),
            },
            FormattedData::FLOAT(val) => match self.datatype_indexes.is_varchar.contains(&self.x_ind) {
                true => batch_set(self, batch, val.to_string()),
                false => batch_set(self, batch, *val),
            },
            FormattedData::DATE(val) => {
                match self.datatype_indexes {
                    ind if ind.is_varchar.contains(&self.x_ind) => batch_set(self, batch, val.to_string()),
                    ind if ind.is_date.contains(&self.x_ind) => batch_set(self, batch, *val),
                    ind if ind.is_int.contains(&self.x_ind) => {
                        let to_num = remove_invalid_chars(&val.to_string());
                        batch_set(self, batch, to_num.parse::<i64>().unwrap())
                    },
                    ind if ind.is_float.contains(&self.x_ind) => {
                        let to_num = remove_invalid_chars(&val.to_string());
                        batch_set(self, batch, to_num.parse::<f64>().unwrap())
                    },
                    _ => batch_set(self, batch, *val),
                }
            },
            FormattedData::EMPTY => {
                match self.datatype_indexes {
                    ind if ind.is_varchar.contains(&self.x_ind) => empty_batch_set!(self, String, batch),
                    ind if ind.is_date.contains(&self.x_ind) => empty_batch_set!(self, chrono::NaiveDateTime, batch),
                    ind if ind.is_int.contains(&self.x_ind) => empty_batch_set!(self, i8, batch),
                    ind if ind.is_float.contains(&self.x_ind) => empty_batch_set!(self, f32, batch),
                    _ => empty_batch_set!(self, String, batch),
                }
            },
        }
    }
}

fn batch_set<T> (cell_props: CellProperties, batch: &mut Batch<'_>, value: T) 
-> Result<(), OracleSqlToolsError>
where T: oracle::sql_type::ToSql + Display {
    match batch.set(cell_props.x_ind + 1, &value) {
        Ok(_) => Ok(()),
        Err(e) => return Err(OracleSqlToolsError::CellPropertyError { 
            error_message: e, 
            cell_value: value.to_string(),
            x_index: cell_props.x_ind, 
            y_index: cell_props.y_ind 
        }),
    }
}