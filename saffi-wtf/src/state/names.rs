use std::fmt;

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use thiserror::Error;
use uuid::Uuid;
use www_saffi::OptionExt as _;

/// The name of a group, either parsed from a raw string or the root group
/// (which has no name).
///
/// Named groups' names can contain only lowercase ASCII-alphabetic characters and dashes.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum GroupName {
    Root,
    Named(String),
}

impl TryFrom<String> for GroupName {
    type Error = ParseGroupNameError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        use ParseGroupNameError::*;

        // Look for any characters that are not lowercase ASCII-alphabetic or dashes. If any are
        // found, this is an invalid group name.
        raw.chars()
            .find(|&c| !(c.is_ascii_lowercase() || c == '-'))
            .map(|inv| InvalidChar(raw.clone(), inv))
            .err_or(GroupName::Named(raw))
    }
}

#[derive(Error, Debug)]
pub enum ParseGroupNameError {
    #[error("group name \"{0}\" contains invalid char '{1}'")]
    InvalidChar(String, char),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TagName(String);

impl TryFrom<String> for TagName {
    type Error = ParseTagNameError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        use ParseTagNameError::*;

        // Look for any characters that are not lowercase ASCII-alphabetic or
        // dashes. If any are found, this is an invalid group name, and the
        // invalid char will be returned in Some().
        raw.chars()
            .find(|&c| !(c.is_ascii_lowercase() || c == '-'))
            .map(|inv| InvalidChar(raw.clone(), inv))
            .err_or(TagName(raw))
    }
}

impl TryFrom<&str> for TagName {
    type Error = ParseTagNameError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::try_from(raw.to_owned())
    }
}

#[derive(Error, Debug)]
pub enum ParseTagNameError {
    #[error("tag name \"{0}\" contains invalid char '{1}'")]
    InvalidChar(String, char),
}

struct TagNameVisitor;

impl<'de> Visitor<'de> for TagNameVisitor {
    type Value = TagName;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("a string containing only lowercase ASCII-alphabetic characters or dashes")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        TagName::try_from(v).map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for TagName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TagNameVisitor)
    }
}

/// The name of a page, either parsed from a raw string or an index page, which
/// has no name beyond the name of the group it's contained in.
///
/// Page names are single path components in a URL, containing only numerals,
/// lowercase ASCII-alphabetic characters and dashes.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PageName {
    Index(Uuid),
    Named(String),
}

impl PageName {
    pub fn new_index() -> Self {
        Self::Index(Uuid::new_v4())
    }
}

impl TryFrom<String> for PageName {
    type Error = ParsePageNameError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        use ParsePageNameError::*;

        // Look for any characters that are not lowercase ASCII-alphabetic or
        // dashes. If any are found, this is an invalid page name, and the
        // invalid char will be returned in Some().
        raw.chars()
            .find(|&c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
            .map(|inv| InvalidChar(raw.clone(), inv))
            .err_or(PageName::Named(raw))
    }
}

impl TryFrom<&str> for PageName {
    type Error = ParsePageNameError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::try_from(raw.to_owned())
    }
}

#[derive(Error, Debug)]
pub enum ParsePageNameError {
    #[error("page name \"{0}\" contains invalid char '{1}'")]
    InvalidChar(String, char),
}
