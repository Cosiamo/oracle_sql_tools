use chrono::NaiveDateTime;
use oracle::Connection;
use types::{FormattedData::{self, DATE, FLOAT, INT, STRING}, PreppedGridData};
pub mod statements;
pub mod types;
pub mod utils;

pub trait FormatData {
    fn fmt_data(self) -> FormattedData;
}

macro_rules! impl_fmt_data {
    ($data:ident, $enum_type:ident) => {
        impl FormatData for $data {
            fn fmt_data(self) -> FormattedData { $enum_type(self.into()) }
        }
    };
}

impl FormatData for FormattedData { fn fmt_data(self) -> FormattedData { self } }
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
impl FormatData for &str { fn fmt_data(self) -> FormattedData { STRING(self.into()) } }
impl_fmt_data!(String, STRING);
impl_fmt_data!(i8, INT);
impl_fmt_data!(i16, INT);
impl_fmt_data!(i32, INT);
impl_fmt_data!(i64, INT);
impl_fmt_data!(f32, FLOAT);
impl_fmt_data!(f64, FLOAT);
impl_fmt_data!(NaiveDateTime, DATE);

// =========================
pub trait PrepData<T: FormatData> {
    fn prep_data(self, conn: Connection) -> PreppedGridData;
}

impl<T: FormatData> PrepData<T> for Vec<Vec<T>> {
    fn prep_data(self, conn: Connection) -> PreppedGridData  {
        let mut grid = Vec::new();
        for row in self {
            let mut inner_vec = Vec::new();
            for cell in row {
                inner_vec.push(cell.fmt_data())
            }
            grid.push(inner_vec)
        }
        PreppedGridData {
            data: grid,
            conn
        }
    }
}