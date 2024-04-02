use crate::{types::FormattedData, utils::remove_invalid_chars};

pub trait MutateRow {
    fn insert_stmt(self, table_name: &str) -> String;
    fn to_string(&self) -> Vec<String>;
}

macro_rules! to_string {
    ($data:ident) => {{
        $data.iter().map(|x| {
            match x {
                FormattedData::STRING(val) => val.to_owned(),
                FormattedData::INT(val) => val.to_string(),
                FormattedData::FLOAT(val) => val.to_string(),
                FormattedData::DATE(val) => val.to_string(),
                FormattedData::EMPTY => "".to_string(),
            }
        }).collect::<Vec<String>>()
    }};
}

impl MutateRow for Vec<FormattedData> {
    fn insert_stmt(self, table_name: &str) -> String {
        // creating indexes in an insert statement for the batch set methods to attach values to
        let mut n: Vec<String> = Vec::new();
        for i in 0..self.len() {
            n.push(remove_invalid_chars(&[":", &(i + 1).to_string()].concat()))
        }
        let header = to_string!(self);
        let insert = [
            "INSERT INTO ", &table_name , " (", 
            &remove_invalid_chars(&header.join(",")), 
            ") VALUES (", &n.join(", "),  &")".to_string()
        ].concat();
        insert
    }
    
    fn to_string(&self) -> Vec<String> { to_string!(self) }
}