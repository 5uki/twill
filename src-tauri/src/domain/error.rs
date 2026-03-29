use serde::Serialize;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppError {
    InvalidCliArgs { message: String },
    UnsupportedFormat { format: String },
    Serialization { message: String },
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCliArgs { message } => write!(f, "CLI 参数错误: {message}"),
            Self::UnsupportedFormat { format } => write!(f, "不支持的输出格式: {format}"),
            Self::Serialization { message } => write!(f, "序列化失败: {message}"),
        }
    }
}

impl Error for AppError {}
