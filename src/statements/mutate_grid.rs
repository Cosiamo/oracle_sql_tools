use oracle::Connection;
use crate::{format_data::FormattedData, types::errors::OracleSqlToolsError};

pub(crate) trait MutateGrid {
    fn replace_header(&mut self, connection: &Connection, table_name: &str) -> Result<(Vec<FormattedData>, &Self), OracleSqlToolsError>;
    fn separate_header(&mut self) -> (Vec<FormattedData>, &Self);
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

    fn divide(&mut self, num: f32) -> Self {
        let res: Vec<_> = self.splice(
            (0)..(num.ceil() as usize - 1),
            []
        ).collect();
        res
    }
}
