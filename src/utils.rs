use crate::types::FormattedData;

pub fn remove_invalid_chars(input: &String) -> String {
    input
        .trim()
        .replace(|c: char| !c.is_ascii(), "")
        .replace(" ", "_")
        .replace("-", "_")
        .replace("'", "")
        .replace("%", "")
        .replace("!", "")
        .replace("?", "")
        .replace("|", "")
        .replace("#", "")
        .replace("\\", "")
        .replace("/", "")
        .replace("(", "")
        .replace(")", "")
        .replace("+", "")
        .replace("#", "")
}

impl FormattedData {
    pub fn to_string(self) -> String {
        match self {
            FormattedData::STRING(val) => val.to_owned(),
            FormattedData::INT(val) => val.to_string(),
            FormattedData::FLOAT(val) => val.to_string(),
            FormattedData::DATE(val) => val.to_string(),
            FormattedData::EMPTY => "".to_string(),
        }
    }
}