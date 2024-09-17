use crate::{types::{errors::OracleSqlToolsError, FilterType}, utils::remove_invalid_chars};
use super::PreppedRowData;

impl PreppedRowData {
    /// Selects the columns (via the input vector) from the specified table.
    /// 
    /// # Usage
    /// 
    /// It's recommended to open a [`Connection`](oracle::Connection) first so if there's an issue connecting to the database, it'll error faster.
    /// 
    /// You'll also need to initialize your column names as a Vector (recommended to use &str or String, but can accept any datatype that implements [`crate::format_data::FormattedData`]).
    /// 
    /// Once the previous two things are done, use the [`.prep_data()`](crate::PrepData::prep_data) method on your vector, then 
    /// ```no_run
    /// let conn: oracle::Connection = match Connection::connect("<USERNAME>", "<PASSWORD>", "<IP ADDRESS>")?; 
    ///
    /// let col_names: Vec<&str> = vec!["Employee ID", "Name", "Job Title", "Department", "Business Unit"];
    ///
    /// let table_data: Vec<Vec<Option<String>>> = col_names.prep_data(conn).select("MY_TABLE")?;
    /// ```
    pub fn select(mut self, table_name: &str) -> Self {
        let header = self.data.iter().map(|cell|
            remove_invalid_chars(cell)
        ).collect::<Vec<String>>();
        let query = format!("SELECT {} FROM {}", &header.join(", "), table_name);
        self.query = Some(query);
        self.header = Some(header);
        self
    }

    pub fn filter(mut self, filter_type: FilterType, column: &str, value: &str) -> Self {
        let ty = match filter_type {
            FilterType::Where => "WHERE",
            FilterType::And => "AND",
            FilterType::Or => "OR"
        };

        let stmt = format!("{} {} = '{}'", ty, column, value);
        match self.filters {
            Some(ref mut val) => val.push(stmt),
            None => self.filters = Some(vec![stmt]),
        };
        self
    }

    pub fn execute(self) -> Result<Vec<Vec<Option<String>>>, OracleSqlToolsError> {
        let header = match self.header {
            Some(val) => val,
            None => return Err(OracleSqlToolsError::SQLQueryEmpty),
        };

        let mut sql = match self.query {
            Some(val) => val,
            None => return Err(OracleSqlToolsError::SQLQueryEmpty),
        };

        match self.filters {
            Some(filters) => {
                let clauses = filters.join(" ");
                sql = format!("{} {}", sql, clauses);
            },
            None => sql = sql,
        }

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