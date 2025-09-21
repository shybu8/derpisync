#[derive(Debug)]
pub enum GeneralError {
    Serde(serde_json::Error),
    Reqwest(reqwest::Error),
}

impl std::fmt::Display for GeneralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serde(a) => a.fmt(f),
            Self::Reqwest(a) => a.fmt(f),
        }
    }
}

impl std::error::Error for GeneralError {}

impl From<serde_json::Error> for GeneralError {
    fn from(value: serde_json::Error) -> Self {
        return GeneralError::Serde(value);
    }
}

impl From<reqwest::Error> for GeneralError {
    fn from(value: reqwest::Error) -> Self {
        return GeneralError::Reqwest(value);
    }
}
