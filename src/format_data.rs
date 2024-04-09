use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub enum FormattedData {
    STRING(String),
    INT(i64),
    FLOAT(f64),
    DATE(NaiveDateTime),
    EMPTY,
}

// A trait that formats the input data to match [`format_data::FormattedData`]
//
// Already implemented for `&[u8]`, `Vec<u8>`, `&str`, 'String', 'i8', 'i16', 'i32', 'i64', 'f32', 'f64', and [`chrono::NaiveDateTime`], as well as, their Option<> variants
//
// To implement a local enum: 
//
// ```no_run
// enum MyEnum {
//     VARCHAR(String),
//     NUMBER(i64)
// }
//
// impl FormatData for MyEnum {
//     fn fmt_data(self) -> FormattedData {
//         match self {
//             MyEnum::VARCHAR(val) => FormattedData::STRING(val.into()),
//             MyEnum::NUMBER(val) => FormattedData::INT(val.into()),
//         }
//     }
// }
// ```
//
// To implement a foreign enum:
//
// ```no_run
// use some_crate::SomeForeignType;
//
// struct MyType<'a>(&'a SomeForeignType);
//
// impl FormatData for MyType<'_> {
//     fn fmt_data(self) -> FormattedData {
//         match self {
//             MyType(SomeForeignType::Int(val)) => FormattedData::INT(*val),
//             MyType(SomeForeignType::Float(val)) => FormattedData::FLOAT(*val),
//             MyType(SomeForeignType::String(val)) => FormattedData::STRING(val.to_owned()),
//             MyType(SomeForeignType::None) => FormattedData::EMPTY,
//         }
//     }
// }
// ```
pub trait FormatData { fn fmt_data(self) -> FormattedData; }

impl FormatData for FormattedData { fn fmt_data(self) -> Self { self } }
impl FormatData for &[u8] {
    fn fmt_data(self) -> FormattedData {
        let clone_on_write_string = String::from_utf8_lossy(self);
        let utf8_string = clone_on_write_string.replace(|c: char| !c.is_ascii(), "");
        FormattedData::STRING(utf8_string)
    }
}
impl FormatData for Vec<u8> {
    fn fmt_data(self) -> FormattedData {
        let utf8_string = String::from_utf8(self)
            .map_err(|non_utf8| 
                String::from_utf8_lossy(non_utf8.as_bytes()).into_owned()
            )
            .unwrap();
        FormattedData::STRING(utf8_string)
    }
}
impl FormatData for Option<&[u8]> {
    fn fmt_data(self) -> FormattedData {
        match self {
            Some(val) => {
                let clone_on_write_string = String::from_utf8_lossy(val);
                let utf8_string = clone_on_write_string.replace(|c: char| !c.is_ascii(), "");
                FormattedData::STRING(utf8_string)
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
                FormattedData::STRING(utf8_string)
            },
            None => FormattedData::EMPTY,
        }
    }
}

macro_rules! impl_fmt_data {
    ($data_type:ty, $enum_type:ident) => {
        impl FormatData for $data_type {
            fn fmt_data(self) -> FormattedData { FormattedData::$enum_type(self.into()) }
        }
    };
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
    ($data_type:ty, $enum_type:ident) => {
        impl FormatData for $data_type {
            fn fmt_data(self) -> FormattedData {
                match self {
                    Some(val) => FormattedData::$enum_type(val.into()),
                    None => FormattedData::EMPTY,
                }
            }
        }
    };
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