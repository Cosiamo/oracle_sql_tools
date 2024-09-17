use crate::statements::{utils::remove_invalid_chars, PreppedRowData};

pub fn get_header_and_query(input: &PreppedRowData, table_name: &str) -> (Vec<String>, String) {
    let header = input.data.iter().map(|cell|
        remove_invalid_chars(cell)
    ).collect::<Vec<String>>();
    let query = format!("SELECT {} FROM {}", &header.join(", "), table_name);
    (header, query)
}