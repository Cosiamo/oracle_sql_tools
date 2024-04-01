use std::{sync::Arc, thread::{self, JoinHandle}};

use crate::types::{errors::OracleSqlToolsError, BatchPrep, CellProperties, FormattedData};

use super::mutate_grid::MutateGrid;


impl BatchPrep {
    pub fn split_batch_by_threads(mut self) -> Result<(), OracleSqlToolsError> {
        let len = self.data_body.len();
        let nthreads = num_cpus::get();
        let num = (len / nthreads + if len % nthreads == 0 { 0 } else { 1 }) as f32;

        let varchar_ind = Arc::new(self.data_indexes.is_varchar);

        // handles captures the spawned threads into a vector
        let mut handles: Vec<JoinHandle<Result<(), OracleSqlToolsError>>> = Vec::new();
        // spawns as many threads as there are on the host CPU
        for n in 0..nthreads {
            let conn = Arc::clone(&self.conn);
            let insert = Arc::clone(&self.insert_stmt);
            let varchar_ind = Arc::clone(&varchar_ind);
            let ad: Arc<Vec<Vec<FormattedData>>>;
            if n + 1 < nthreads {
                // splits up the 2d vector to have an even amount per thread
                let d = self.data_body.divide(num);
                // new ARC per each split vector
                ad = Arc::new(d);
            } else {
                // collecting the remaining data
                ad = Arc::new(self.data_body.to_owned());
            }
            handles.push(thread::spawn(move || {
                let batch: oracle::Batch<'_> = match conn.batch(&insert.as_str(), ad.len()).build() {
                    Ok(val) => val,
                    Err(e) => return Err(OracleSqlToolsError::OracleError(e)),
                };
                match ad.handle_concurrency((num.ceil() as usize - 1) * n, varchar_ind, batch) {
                    Ok(val) => return Ok(val),
                    Err(e) => return Err(e),
                };
            }));
        }
        // executes all threads
        for handle in handles {
            handle.join().unwrap()?;
        }
        self.conn.execute("ALTER SESSION SET NLS_DATE_FORMAT='MM/DD/YYYY HH:MI:SS AM'", &[])?;
        self.conn.commit()?;
        Ok(())
    }
}

pub trait AtomicReffedData {
    fn handle_concurrency(
        self, num: usize, varchar_ind: Arc<Vec<usize>>, batch: oracle::Batch<'_>
    ) -> Result<(), OracleSqlToolsError>;
}

impl AtomicReffedData for Arc<Vec<Vec<FormattedData>>> {
    fn handle_concurrency(
        self, num: usize, varchar_ind: Arc<Vec<usize>>, mut batch: oracle::Batch<'_>
    ) -> Result<(), OracleSqlToolsError> {
        self.iter().enumerate().try_for_each(|(y, row)| -> Result<(), OracleSqlToolsError> {
            row.iter().enumerate().try_for_each(|(x, cell)| -> Result<(), OracleSqlToolsError> {
                CellProperties {
                    cell,
                    varchar_ind: varchar_ind.clone(),
                    x_ind: x,
                    y_ind: (num + y),
                }.match_stmt(&mut batch)
            })?;
            batch.append_row(&[])?;
            Ok(())
        })?;
        batch.execute()?;
        Ok(())
    }
}

impl CellProperties<'_> {
    pub fn match_stmt(self, batch: &mut oracle::Batch<'_>) -> Result<(), OracleSqlToolsError> {
        match self.cell {
            FormattedData::STRING(val) => Ok(batch.set(self.x_ind + 1, val)?),
            FormattedData::INT(val) => {
                match self.varchar_ind.contains(&self.x_ind) {
                    true => Ok(batch.set(self.x_ind + 1, &val.to_string())?),
                    false => Ok(batch.set(self.x_ind + 1, val)?),
                }
            },
            FormattedData::FLOAT(val) => {
                match self.varchar_ind.contains(&self.x_ind) {
                    true => Ok(batch.set(self.x_ind + 1, &val.to_string())?),
                    false => Ok(batch.set(self.x_ind + 1, val)?),
                }
            },
            FormattedData::DATE(val) => {
                let stamp = match val.to_string().parse::<String>() {
                    Ok(stamp) => stamp,
                    Err(e) => panic!("{:?}", e),
                };
                Ok(batch.set(self.x_ind + 1, &stamp)?)
            },
            FormattedData::EMPTY => Ok(batch.set(self.x_ind + 1, &None::<String>)?),
        }
    }
}