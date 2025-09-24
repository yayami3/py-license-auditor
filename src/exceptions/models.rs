use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exception {
    pub name: String,
    pub version: Option<String>,
    pub reason: String,
    pub added_by: Option<String>,
    pub added_date: DateTime<Utc>,
    pub expires: Option<NaiveDate>,
    pub permanent: bool,
    pub added_interactively: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExceptionsFile {
    pub exceptions: Vec<Exception>,
}

impl ExceptionsFile {
    pub fn new() -> Self {
        Self {
            exceptions: Vec::new(),
        }
    }

    pub fn add_exception(&mut self, exception: Exception) {
        self.exceptions.push(exception);
    }
}
