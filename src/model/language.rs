use crate::error;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Language {
    Unsupported,
    Cpp = 1,
    Java = 4,
    Bash = 10,
    CppWithFileSupport = 12,
    // C = ?,
    Ada = 9,
}

impl Language {
    pub fn code(&self) -> String {
        let val = *self;
        let val = val as i32;
        val.to_string()
    }
}

impl ToString for Language {
    fn to_string(&self) -> String {
        match self {
            Language::Cpp => "C++",
            Language::Java => "Java",
            Language::Bash => "Bash",
            Language::CppWithFileSupport => "C++ with file support",
            Language::Ada => "Ada",
            Language::Unsupported => "Unsupported language",
        }
        .to_string()
    }
}

impl FromStr for Language {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let l = match s.to_lowercase().as_str() {
            "c++" => Language::Cpp,
            "java" => Language::Java,
            "bash" => Language::Bash,
            "c++ z obsluga plikow" => Language::CppWithFileSupport,
            "ada" => Language::Ada,
            lang => return Err(Self::Err::UnsupportedLanguage(lang.to_string())),
        };

        Ok(l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn from_string() {
        assert_eq!(Language::from_str("C++").unwrap(), Language::Cpp);
        assert_eq!(Language::from_str("Java").unwrap(), Language::Java);
        assert_eq!(Language::from_str("Bash").unwrap(), Language::Bash);
        assert_eq!(
            Language::from_str("C++ z obsluga plikow").unwrap(),
            Language::CppWithFileSupport
        );
        assert_eq!(Language::from_str("Ada").unwrap(), Language::Ada);
        assert!(Language::from_str("C").is_err());
    }

    #[test]
    fn to_string() {
        assert_eq!(Language::Unsupported.to_string(), "Unsupported language");
        assert_eq!(Language::Cpp.to_string(), "C++");
        assert_eq!(Language::Java.to_string(), "Java");
        assert_eq!(Language::Bash.to_string(), "Bash");
        assert_eq!(
            Language::CppWithFileSupport.to_string(),
            "C++ with file support"
        );
        assert_eq!(Language::Ada.to_string(), "Ada");
    }

    #[test]
    fn from_string_polish() {
        let result = Language::from_str("C++ z obsługą plików");
        assert!(matches!(result, Err(Error::UnsupportedLanguage { .. })));
    }

    #[test]
    fn from_invalid_string() {
        let result = Language::from_str("sada224214@dasdas");
        assert!(matches!(result, Err(Error::UnsupportedLanguage { .. })));
    }

    #[test]
    fn from_different_case_string() {
        assert_eq!(
            Language::from_str("c++ z Obsluga pliKOW").unwrap(),
            Language::CppWithFileSupport
        );
    }

    #[test]
    fn codes() {
        assert_eq!(Language::Unsupported.code(), "0");
        assert_eq!(Language::Cpp.code(), "1");
        assert_eq!(Language::Java.code(), "4");
        assert_eq!(Language::Bash.code(), "10");
        assert_eq!(Language::CppWithFileSupport.code(), "12");
        assert_eq!(Language::Ada.code(), "9");
    }
}
