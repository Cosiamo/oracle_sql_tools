use oracle::Connection;
use crate::{format_data::FormattedData, types::{errors::OracleSqlToolsError, DatatypeIndexes}};

pub(crate) trait MutateGrid {
    fn replace_header(&mut self, connection: &Connection, table_name: &str) -> Result<(Vec<FormattedData>, &Self), OracleSqlToolsError>;
    fn separate_header(&mut self) -> (Vec<FormattedData>, &Self);
    fn get_col_datatype(&self) -> DatatypeIndexes;
    fn get_varchar_ind(&self) -> DatatypeIndexes;
    fn divide(&mut self, num: f32) -> Self;
}

impl MutateGrid for Vec<Vec<FormattedData>> {
    fn replace_header(&mut self, connection: &Connection, table_name: &str) -> Result<(Vec<FormattedData>, &Self), OracleSqlToolsError> {
        let mut header: Vec<FormattedData> = Vec::new();
        let sql = [
            "select COLUMN_NAME from ALL_TAB_COLUMNS where lower(TABLE_NAME)='".to_string(), 
            table_name.to_ascii_lowercase(), "'".to_string()
        ].concat();
        let rows = connection.query(&sql, &[])?;
        for row_result in rows {
            let row = row_result?;
            for val in row.sql_values() {
                let t = val.get()?;
                header.push(FormattedData::STRING(t))
            }
        }
        let fmt_header = header.into();
        self.splice((0)..(1), []);
        Ok((fmt_header, self))
    }
    
    fn separate_header(&mut self) -> (Vec<FormattedData>, &Self) {
        // let mut header = Vec::new();
        let header: Vec<_> = self.splice((0)..(1), []).collect();
        let res = &header[0];
        (res.to_owned(), self)
    }
    
    fn get_col_datatype(&self) -> DatatypeIndexes {
        let mut is_varchar: Vec<usize> = Vec::new();
        let mut is_float: Vec<usize> = Vec::new();
        let mut is_int: Vec<usize> = Vec::new();
        let mut is_date: Vec<usize> = Vec::new();
        // find varchar
        'row: for x in 0..self[0].len() {
            for y in 1..self.len() {
                match &self[y][x] {
                    FormattedData::STRING(_) => { is_varchar.push(x); continue 'row; },
                    FormattedData::INT(_) => continue,
                    FormattedData::FLOAT(_) => continue,
                    FormattedData::DATE(_) => continue,
                    FormattedData::EMPTY => continue,
                }
            }
        }
        // find float
        'row: for x in 0..self[0].len() {
            if is_varchar.contains(&x) { continue }
            for y in 1..self.len() {
                match &self[y][x] {
                    FormattedData::STRING(_) => continue,
                    FormattedData::INT(_) => continue,
                    FormattedData::FLOAT(_) => { is_float.push(x); continue 'row; },
                    FormattedData::DATE(_) => continue,
                    FormattedData::EMPTY => continue,
                }
            }
        }
        // find int
        'row: for x in 0..self[0].len() {
            if is_varchar.contains(&x) || is_float.contains(&x) { continue }
            for y in 1..self.len() {
                match &self[y][x] {
                    FormattedData::STRING(_) => continue,
                    FormattedData::INT(_) => { is_int.push(x); continue 'row; },
                    FormattedData::FLOAT(_) => continue,
                    FormattedData::DATE(_) => continue,
                    FormattedData::EMPTY => continue,
                }
            }
        }
        // find date
        'row: for x in 0..self[0].len() {
            if is_varchar.contains(&x) || is_float.contains(&x) || is_int.contains(&x) { continue }
            for y in 1..self.len() {
                match &self[y][x] {
                    FormattedData::STRING(_) => continue,
                    FormattedData::INT(_) => continue,
                    FormattedData::FLOAT(_) => continue,
                    FormattedData::DATE(_) => { is_date.push(x); continue 'row; },
                    FormattedData::EMPTY => continue,
                }
            }
        }

        DatatypeIndexes {
            is_varchar,
            is_float,
            is_int,
            is_date,
        }
    }

    fn get_varchar_ind(&self) -> DatatypeIndexes {
        let mut is_varchar: Vec<usize> = Vec::new();
        let is_float: Vec<usize> = Vec::new();
        let is_int: Vec<usize> = Vec::new();
        let is_date: Vec<usize> = Vec::new();
        // find varchar
        'row: for x in 0..self[0].len() {
            for y in 1..self.len() {
                match &self[y][x] {
                    FormattedData::STRING(_) => { is_varchar.push(x); continue 'row; },
                    FormattedData::INT(_) => continue,
                    FormattedData::FLOAT(_) => continue,
                    FormattedData::DATE(_) => continue,
                    FormattedData::EMPTY => continue,
                }
            }
        }
        DatatypeIndexes {
            is_varchar,
            is_float,
            is_int,
            is_date,
        }
    }

    fn divide(&mut self, num: f32) -> Self {
        let res: Vec<_> = self.splice(
            (0)..(num.ceil() as usize - 1),
            []
        ).collect();
        res
    }
}
