use std::{sync::Arc, thread::{self, JoinHandle}};
use oracle::Connection;

use crate::types::{errors::OracleSqlToolsError, AtomicReffedData, BatchPrep, CellProperties, FormattedData};
use super::mutate_grid::MutateGrid;

macro_rules! iterate_grid {
    ($input:expr, $varchar_ind:ident, $num:ident, $batch:ident) => {{
        // each thread iterates over their slice of the data
        $input.data.iter().enumerate().try_for_each(|(y, row)| 
        -> Result<(), OracleSqlToolsError> {
            row.iter().enumerate().try_for_each(|(x, cell)| 
            -> Result<(), OracleSqlToolsError> {
                CellProperties {
                    cell,
                    varchar_ind: &$varchar_ind,
                    x_ind: x,
                    y_ind: ($num + y),
                }.match_stmt(&mut $batch)
            })?;
            $batch.append_row(&[])?;
            Ok(())
        })?;
        $batch.execute()?;
        Ok(())
    }};
}

impl BatchPrep {
    pub fn split_batch_by_threads(mut self) -> Result<(), OracleSqlToolsError> {
        // Atomically Reference Counted the connection and the insert statement 
        let conn: Arc<Connection> = Arc::new(self.conn);
        let insert_stmt: Arc<String> = Arc::new(self.insert_stmt);

        // divides the length of the data by the number of threads on the host CPU
        let len = self.data.len();
        let nthreads = num_cpus::get();
        let num = (len / nthreads + if len % nthreads == 0 { 0 } else { 1 }) as f32;

        let varchar_ind = Arc::new(self.data_indexes.is_varchar);
        // captures the spawned threads into a vector
        let mut handles: Vec<JoinHandle<Result<(), OracleSqlToolsError>>> = Vec::new();
        // iterates as many times as there are threads
        for n in 0..nthreads {
            // each thread needs to have it's own clone of the data
            let conn = Arc::clone(&conn);
            let insert = Arc::clone(&insert_stmt);
            let varchar_ind = Arc::clone(&varchar_ind);
            let ad: Arc<Vec<Vec<FormattedData>>>;
            if n + 1 < nthreads {
                // splits up the 2d vector to have an even amount per thread
                let d = self.data.divide(num);
                // new ARC per each split vector
                ad = Arc::new(d);
            // collecting the remaining data
            } else { ad = Arc::new(self.data.to_owned()); }
            handles.push(thread::spawn(move || {
                // creates a unique Batch per thread
                let batch: oracle::Batch<'_> = match conn.batch(&insert.as_str(), ad.len()).build() {
                    Ok(val) => val,
                    Err(e) => return Err(OracleSqlToolsError::OracleError(e)),
                };
                let info = AtomicReffedData {
                    data: ad,
                    num: (num.ceil() as usize - 1) * n,
                    varchar_ind,
                };
                match info.handle_concurrency(batch) {
                    Ok(val) => return Ok(val),
                    Err(e) => return Err(e),
                };
            }));
        }
        // executes all threads
        for handle in handles {
            handle.join().unwrap()?;
        }
        conn.commit()?;
        Ok(())
    }

    pub fn single_thread_batch(self) -> Result<(), OracleSqlToolsError> {
        let body_len = &self.data.len();
        let mut batch: oracle::Batch<'_> = match self.conn.batch(&self.insert_stmt.as_str(), body_len.to_owned()).build() {
            Ok(val) => val,
            Err(e) => return Err(OracleSqlToolsError::OracleError(e)),
        };
        // CellProperties expects an Arc<Vec<usize>>
        let varchar_ind = Arc::new(self.data_indexes.is_varchar);
        // num is 0 because we're not dividing by any threads
        let num = 0 as usize;
        match iterate_grid!(self, varchar_ind, num, batch) {
            Ok(_) => {},
            Err(e) => return Err(e),
        };
        self.conn.commit()?;
        Ok(())
    }
}

impl AtomicReffedData {
    fn handle_concurrency(
        self, mut batch: oracle::Batch<'_>
    ) -> Result<(), OracleSqlToolsError> {
        let varchar_ind = self.varchar_ind;
        let num = self.num;
        iterate_grid!(self, varchar_ind, num, batch)
    }
}

macro_rules! batch_set {
    ($data:ident, $batch:ident, $val:ident) => {{
        match $batch.set($data.x_ind + 1, $val) {
            Ok(val) => return Ok(val),
            Err(e) => return Err(OracleSqlToolsError::CellPropertyError { 
                error_message: e, 
                cell_value: $val.to_string(),
                x_index: $data.x_ind, 
                y_index: $data.y_ind 
            }),
        }
    }};
}

impl CellProperties<'_> {
    pub fn match_stmt(self, batch: &mut oracle::Batch<'_>) -> Result<(), OracleSqlToolsError> {
        match self.cell {
            FormattedData::STRING(val) => batch_set!(self, batch, val),
            FormattedData::INT(val) => {
                match self.varchar_ind.contains(&self.x_ind) {
                    true => { 
                        let borrowed_val = &val.to_string(); 
                        batch_set!(self, batch, borrowed_val);
                    },
                    false => batch_set!(self, batch, val),
                }
            },
            FormattedData::FLOAT(val) => {
                match self.varchar_ind.contains(&self.x_ind) {
                    true => { 
                        let borrowed_val = &val.to_string(); 
                        batch_set!(self, batch, borrowed_val);
                    },
                    false => batch_set!(self, batch, val),
                }
            },
            FormattedData::DATE(val) => {
                let stamp = val.to_string().parse::<String>()?;
                let res = &stamp;
                batch_set!(self, batch, res)
            },
            FormattedData::EMPTY => {
                match batch.set(self.x_ind + 1, &None::<String>) {
                    Ok(val) => return Ok(val),
                    Err(e) => return Err(OracleSqlToolsError::CellPropertyError { 
                        error_message: e, 
                        cell_value: "NULL".to_string(),
                        x_index: self.x_ind, 
                        y_index: self.y_ind 
                    }),
                }
            },
        }
    }
}