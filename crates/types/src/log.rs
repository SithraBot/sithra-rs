use std::fmt::Display;

use log::Level;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub level:       Level,
    pub message:     String,
    pub target:      String,
    pub module_path: Option<String>,
    pub loc_file:    Option<String>,
    pub loc_line:    Option<u32>,
    pub kvs:         Vec<(String, Value)>,
}

impl Log {
    pub fn log(&self) {
        log::logger().log(
            &log::Record::builder()
                .args(format_args!("{}", self.message))
                .level(self.level)
                .target(&self.target)
                .module_path(self.module_path.as_deref())
                .file(self.loc_file.as_deref())
                .line(self.loc_line)
                .key_values(&self.kvs)
                .build(),
        );
    }
}

impl From<log::Record<'_>> for Log {
    fn from(value: log::Record<'_>) -> Self {
        let mut visitor = KvVisitor::default();
        value.key_values().visit(&mut visitor).ok();
        Self {
            level:       value.level(),
            message:     value.args().to_string(),
            target:      value.target().to_owned(),
            module_path: value.module_path().map(str::to_owned),
            loc_file:    value.file().map(str::to_owned),
            loc_line:    value.line(),
            kvs:         visitor.into_inner(),
        }
    }
}

impl From<&log::Record<'_>> for Log {
    fn from(value: &log::Record<'_>) -> Self {
        let mut visitor = KvVisitor::default();
        value.key_values().visit(&mut visitor).ok();
        Self {
            level:       value.level(),
            message:     value.args().to_string(),
            target:      value.target().to_owned(),
            module_path: value.module_path().map(str::to_owned),
            loc_file:    value.file().map(str::to_owned),
            loc_line:    value.line(),
            kvs:         visitor.into_inner(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Value {
    None,
    Bool(bool),
    Char(char),
    I64(i64),
    U64(u64),
    F64(f64),
    I128(i128),
    U128(u128),
    String(String),
    Error(ErrorString),
}

impl log::kv::ToValue for Value {
    fn to_value(&self) -> log::kv::Value {
        match self {
            Self::None => log::kv::Value::null(),
            Self::Bool(b) => log::kv::Value::from(*b),
            Self::Char(c) => log::kv::Value::from(*c),
            Self::I64(i) => log::kv::Value::from(*i),
            Self::U64(u) => log::kv::Value::from(*u),
            Self::F64(f) => log::kv::Value::from(*f),
            Self::I128(i) => log::kv::Value::from(*i),
            Self::U128(u) => log::kv::Value::from(*u),
            Self::String(s) => log::kv::Value::from(s.as_str()),
            Self::Error(e) => log::kv::Value::from_dyn_error(e),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorString(String);
impl From<String> for ErrorString {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl Display for ErrorString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ErrorString {}

impl Default for Value {
    fn default() -> Self {
        Self::None
    }
}

impl log::kv::VisitValue<'_> for Value {
    fn visit_any(&mut self, _value: log::kv::Value) -> Result<(), log::kv::Error> {
        Ok(())
    }

    fn visit_null(&mut self) -> Result<(), log::kv::Error> {
        *self = Self::None;
        Ok(())
    }

    fn visit_u64(&mut self, value: u64) -> Result<(), log::kv::Error> {
        *self = Self::U64(value);
        Ok(())
    }

    fn visit_i64(&mut self, value: i64) -> Result<(), log::kv::Error> {
        *self = Self::I64(value);
        Ok(())
    }

    fn visit_u128(&mut self, value: u128) -> Result<(), log::kv::Error> {
        *self = Self::U128(value);
        Ok(())
    }

    fn visit_i128(&mut self, value: i128) -> Result<(), log::kv::Error> {
        *self = Self::I128(value);
        Ok(())
    }

    fn visit_f64(&mut self, value: f64) -> Result<(), log::kv::Error> {
        *self = Self::F64(value);
        Ok(())
    }

    fn visit_bool(&mut self, value: bool) -> Result<(), log::kv::Error> {
        *self = Self::Bool(value);
        Ok(())
    }

    fn visit_str(&mut self, value: &str) -> Result<(), log::kv::Error> {
        *self = Self::String(value.to_owned());
        Ok(())
    }

    fn visit_borrowed_str(&mut self, value: &'_ str) -> Result<(), log::kv::Error> {
        self.visit_str(value)
    }

    fn visit_char(&mut self, value: char) -> Result<(), log::kv::Error> {
        let mut b = [0; 4];
        self.visit_str(&*value.encode_utf8(&mut b))
    }

    fn visit_error(
        &mut self,
        err: &(dyn std::error::Error + 'static),
    ) -> Result<(), log::kv::Error> {
        *self = Self::Error(err.to_string().into());
        Ok(())
    }

    fn visit_borrowed_error(
        &mut self,
        err: &'_ (dyn std::error::Error + 'static),
    ) -> Result<(), log::kv::Error> {
        *self = Self::Error(err.to_string().into());
        Ok(())
    }
}

impl From<log::kv::Value<'_>> for Value {
    fn from(value: log::kv::Value<'_>) -> Self {
        let mut this = Self::default();
        value.visit(&mut this).ok();
        this
    }
}

#[derive(Debug, Default)]
struct KvVisitor(Vec<(String, Value)>);

impl<'kvs> log::kv::VisitSource<'kvs> for KvVisitor {
    fn visit_pair(
        &mut self,
        key: log::kv::Key<'kvs>,
        value: log::kv::Value<'kvs>,
    ) -> Result<(), log::kv::Error> {
        self.0.push((key.as_str().to_owned(), Value::from(value)));
        Ok(())
    }
}
impl KvVisitor {
    fn into_inner(self) -> Vec<(String, Value)> {
        self.0
    }
}

pub mod command {
    use sithra_server::typed;

    use super::Log;
    use crate::{into_request, into_response};

    typed!("/log.create" => impl Log);
    into_response!("/log.create", Log);
    into_request!("/log.create", Log);
}
