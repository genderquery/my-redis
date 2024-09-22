use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Value {
    Ok,
    Nil,
    ServerError(String),
    SimpleString(String),
    BulkString(Vec<u8>),
    Integer(i64),
    Array(Vec<Value>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_value_debug(f, self, 0)
    }
}

fn write_value_debug(f: &mut fmt::Formatter<'_>, v: &Value, indent: usize) -> fmt::Result {
    match v {
        Value::Ok => writeln!(f, "OK"),
        Value::Nil => writeln!(f, "(nil)"),
        Value::ServerError(s) => writeln!(f, "(error) {s}"),
        Value::SimpleString(s) => writeln!(f, "\"{s}\""),
        Value::BulkString(b) => writeln!(f, "\"{}\"", b.escape_ascii()),
        Value::Integer(i) => writeln!(f, "(integer) {i}"),
        Value::Array(v) => {
            if v.is_empty() {
                writeln!(f, "(empty array)")?;
            } else {
                let num_width = (v.len() as f64).log10().floor() as usize + 1;
                for (i, v) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, "{:>indent$}", "")?;
                    }
                    write!(f, "{i:>num_width$}) ", i = i + 1)?;
                    write_value_debug(f, v, indent + num_width + 2)?
                }
            }
            Ok(())
        }
    }
}
