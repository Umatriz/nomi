use regex::Regex;
use thiserror::Error;
#[derive(Debug, PartialEq)]
pub struct Username(String);

impl Default for Username {
    fn default() -> Self {
        Self(String::from("Nomi"))
    }
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid username form\nThe username cannot be more than 16 letters or less than 3\nYou may use:\nA-Z characters, a-z characters, 0-9 numbers, `_` (underscore) symbol")]
    InvalidUsername,
}

impl Username {
    pub fn new(s: impl Into<String>) -> anyhow::Result<Self> {
        let s = s.into();
        let re = Regex::new(r"^[a-zA-Z0-9_]{3,16}$")?;
        match re.captures(&s) {
            Some(_) => Ok(Username(s)),
            None => Err(ValidationError::InvalidUsername.into()),
        }
    }

    pub fn get(&self) -> &String {
        &self.0
    }
}
