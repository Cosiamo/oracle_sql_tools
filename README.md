# Oracle SQL Tools


A crate that makes simple Oracle SQL queries easy to implement into your codebase. Built as an extension to the [Rust-oracle](https://crates.io/crates/oracle) crate (required).

## How To Use

### Set Up Dependencies
Add these inside your `cargo.toml` file:
```toml
[dependencies]
oracle_sql_tools = "0.1"
oracle = "0.5"
# chrono is required if you're working with dates 
chrono = "0.4"
```

### Implement `FormatData` Trait for Local Enums
To use the `.prep_data()` method on a vector or grid that uses an enum you created as the values,  you need to implement the trait `FormatData` for it.
```rust
enum MyEnum {
    VARCHAR(String),
    NUMBER(i64)
}

impl FormatData for MyEnum {
    fn fmt_data(self) -> FormattedData {
        match self {
            MyEnum::VARCHAR(val) => FormattedData::STRING(val.into()),
            MyEnum::NUMBER(val) => FormattedData::INT(val.into()),
        }
    }
}
```

### Implement `FormatData` Trait for Foreign Enums
If you need to implement the trait on a enum from a crate you imported:
```rust
use some_crate::SomeForeignType;

struct MyType<'a>(&'a SomeForeignType);

impl FormatData for MyType<'_> {
    fn fmt_data(self) -> FormattedData {
        match self.0 {
            MyType(SomeForeignType::Int(val)) => FormattedData::INT(*val),
            MyType(SomeForeignType::Float(val)) => FormattedData::FLOAT(*val),
            MyType(SomeForeignType::String(val)) => FormattedData::STRING(val.to_owned()),
            MyType(SomeForeignType::None) => FormattedData::EMPTY,
        }
    }
}
```

## Examples

### Select
``` rust
let conn: oracle::Connection = match Connection::connect("<USERNAME>", "<PASSWORD>", "<IP ADDRESS>")?; 

let col_names: Vec<&str> = vec!["Employee ID", "Name", "Job Title", "Department", "Business Unit"];

let table_data: Vec<Vec<Option<String>>> = col_names.prep_data(conn).select("MY_TABLE")?;
```
Is the same as:
```sql
SELECT employee_id, name, job_title, department, business_unit FROM my_table;
```

### Insert
```rust
let conn: oracle::Connection = match Connection::connect("<USERNAME>", "<PASSWORD>", "<IP ADDRESS>")?; 

let data: Vec<Vec<String>> = vec![
    vec!["ColA".to_string(), "ColB".to_string(), "ColC".to_string()],
    vec!["A1".to_string(), "B1".to_string(), "C1".to_string()],
    vec!["A2".to_string(), "B2".to_string(), "C2".to_string()],
    vec!["A3".to_string(), "B3".to_string(), "C3".to_string()],
];

let res: Arc<Connection> = data.prep_data(conn).insert("MY_TABLE")?;
// `res` is Atomically Referencing `conn`
// `res` has the executed Batch(es), you only need to commit it
res.commit()?;
Ok(())
```
Is the same as:
```sql
INSERT ALL
    INTO my_table (ColA, ColB, ColC) VALUES ('A1', 'B1', 'C1')
    INTO my_table (ColA, ColB, ColC) VALUES ('A2', 'B2', 'C2')
    INTO my_table (ColA, ColB, ColC) VALUES ('A3', 'B3', 'C3')
SELECT 1 FROM dual;
```
Or in `Oracle 23c`:
```sql
INSERT INTO my_table (ColA, ColB, ColC) VALUES 
('A1', 'B1', 'C1'),
('A2', 'B2', 'C2'),
('A3', 'B3', 'C3');
```