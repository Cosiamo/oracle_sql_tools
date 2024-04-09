use crate::{types::{errors::OracleSqlToolsError, PreppedRowData}, utils::remove_invalid_chars};

impl PreppedRowData {
    pub fn select(self, table_name: &str) -> Result<Vec<Vec<Option<String>>>, OracleSqlToolsError> {
        let header = self.data.iter().map(|cell|
            remove_invalid_chars(cell)
        ).collect::<Vec<String>>();
        let sql = format!("SELECT {} FROM {}", &header.join(", "), table_name);

        let query = self.conn.query(&sql, &[])?;
        let mut outer_vec = Vec::new();
        for v in query {
            let p = v?;
            let mut inner_vec = Vec::new();
            for colindx in 0..header.len() {
                let a = p.get::<usize, Option<String>>(colindx)?;
                inner_vec.push(a)
            }
            outer_vec.push(inner_vec)
        }

        Ok(outer_vec)
    }
}