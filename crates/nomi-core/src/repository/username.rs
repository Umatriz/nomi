use regex::Regex;
use serde::{de::Visitor, Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct Username(String);

impl Default for Username {
    fn default() -> Self {
        Self(String::from("Nomi"))
    }
}

struct UsernameVisitor;

impl<'de> Visitor<'de> for UsernameVisitor {
    type Value = Username;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v.as_str())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match Username::new(v) {
            Ok(u) => Ok(u),
            Err(e) => Err(E::custom(format!("{e:#?}"))),
        }
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UsernameVisitor)
    }
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error(
        "
    Invalid username form
    The username cannot be more than 16 letters or less than 3
    You may use:
    A-Z characters, a-z characters, 0-9 numbers, `_` (underscore) symbol
    "
    )]
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

    #[must_use]
    pub fn get(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct Wrap {
        username: Username,
    }

    #[test]
    fn serialize_test() {
        let u = Username::new("ssd").unwrap();
        let toml = toml::to_string_pretty(&Wrap { username: u }).unwrap();
        println!("{toml}");
    }

    #[test]
    fn deserialize_test() {
        let s = "username = \"ssd\"";
        let toml = toml::from_str::<Wrap>(s).unwrap();
        println!("{toml:#?}");
    }
}
