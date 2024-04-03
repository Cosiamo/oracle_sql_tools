use chrono::NaiveDateTime;
use oracle::Connection;
use types::{FormattedData::{self, DATE, FLOAT, INT, STRING}, PreppedGridData};
pub mod statements;
pub mod types;
pub mod utils;

pub trait FormatData {
    fn fmt_data(self) -> FormattedData;
}
impl FormatData for FormattedData { fn fmt_data(self) -> FormattedData { self } }

macro_rules! impl_fmt_data {
    ($data:ty, $enum_type:ident) => {
        impl FormatData for $data {
            fn fmt_data(self) -> FormattedData { $enum_type(self.into()) }
        }
    };
}

impl FormatData for &[u8] {
    fn fmt_data(self) -> FormattedData {
        let clone_on_write_string = String::from_utf8_lossy(self);
        let utf8_string = clone_on_write_string.replace(|c: char| !c.is_ascii(), "");
        STRING(utf8_string)
    }
}
impl FormatData for Vec<u8> {
    fn fmt_data(self) -> FormattedData {
        let utf8_string = String::from_utf8(self)
            .map_err(|non_utf8| 
                String::from_utf8_lossy(non_utf8.as_bytes()).into_owned()
            )
            .unwrap();
        STRING(utf8_string)
    }
}
impl_fmt_data!(&str, STRING);
impl_fmt_data!(String, STRING);
impl_fmt_data!(i8, INT);
impl_fmt_data!(i16, INT);
impl_fmt_data!(i32, INT);
impl_fmt_data!(i64, INT);
impl_fmt_data!(f32, FLOAT);
impl_fmt_data!(f64, FLOAT);
impl_fmt_data!(NaiveDateTime, DATE);

macro_rules! impl_fmt_data_option {
    ($data:ty, $enum_type:ident) => {
        impl FormatData for $data {
            fn fmt_data(self) -> FormattedData {
                match self {
                    Some(val) => $enum_type(val.into()),
                    None => FormattedData::EMPTY,
                }
            }
        }
    };
}

impl FormatData for Option<&[u8]> {
    fn fmt_data(self) -> FormattedData {
        match self {
            Some(val) => {
                let clone_on_write_string = String::from_utf8_lossy(val);
                let utf8_string = clone_on_write_string.replace(|c: char| !c.is_ascii(), "");
                STRING(utf8_string)
            },
            None => FormattedData::EMPTY,
        }
    }
}
impl FormatData for Option<Vec<u8>> {
    fn fmt_data(self) -> FormattedData {
        match self {
            Some(val) => {
                let utf8_string = String::from_utf8(val)
                    .map_err(|non_utf8| 
                        String::from_utf8_lossy(non_utf8.as_bytes()).into_owned()
                    )
                    .unwrap();
                STRING(utf8_string)
            },
            None => FormattedData::EMPTY,
        }
    }
}
impl_fmt_data_option!(Option<&str>, STRING);
impl_fmt_data_option!(Option<String>, STRING);
impl_fmt_data_option!(Option<i8>, INT);
impl_fmt_data_option!(Option<i16>, INT);
impl_fmt_data_option!(Option<i32>, INT);
impl_fmt_data_option!(Option<i64>, INT);
impl_fmt_data_option!(Option<f32>, FLOAT);
impl_fmt_data_option!(Option<f64>, FLOAT);
impl_fmt_data_option!(Option<NaiveDateTime>, DATE);

// =========================
pub trait PrepData<T: FormatData> {
    type Prep;

    fn prep_data(self, connection: Connection) -> Self::Prep;
}

impl<T: FormatData> PrepData<T> for Vec<Vec<T>> {
    type Prep = PreppedGridData;

    fn prep_data(self, connection: Connection) -> Self::Prep  {
        let mut grid = Vec::new();
        for row in self {
            let mut inner_vec = Vec::new();
            for cell in row {
                inner_vec.push(cell.fmt_data())
            }
            grid.push(inner_vec)
        }
        Self::Prep {
            data: grid,
            conn: connection
        }
    }
}