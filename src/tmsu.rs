use std::process::{Command, Stdio};

#[derive(Debug)]
pub enum TmsuError {
    NoExec,
    NoDB,
    ChildErr,
    TaggingErr,
}

impl std::fmt::Display for TmsuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoExec => write!(f, "ERROR: Tmsu executable not found"),
            Self::NoDB => write!(f, "ERROR: Tmsu database not found"),
            Self::ChildErr => write!(f, "ERROR: Tmsu exited unexpectedly"),
            Self::TaggingErr => write!(f, "ERROR: Tagging process was unsuccessful"),
        }
    }
}

impl std::error::Error for TmsuError {}

pub fn test_tmsu() -> Result<(), TmsuError> {
    let tmsu_ver_status = Command::new("tmsu")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .map_err(|_| TmsuError::ChildErr)?;
    if !tmsu_ver_status.success() {
        return Err(TmsuError::NoExec);
    }

    let tmsu_info_status = Command::new("tmsu")
        .arg("info")
        .stdout(Stdio::null())
        .status()
        .map_err(|_| TmsuError::ChildErr)?;
    if !tmsu_info_status.success() {
        return Err(TmsuError::NoDB);
    }
    Ok(())
}

pub fn tag_file(file: &str, tags: impl IntoIterator<Item = String>) -> Result<(), TmsuError> {
    let mut tmsu_tag = Command::new("tmsu");
    tmsu_tag.arg("tag");
    tmsu_tag.arg(file);
    tmsu_tag.args(tags);
    tmsu_tag.stdout(Stdio::null());

    let tmsu_tag_status = tmsu_tag.status().map_err(|_| TmsuError::ChildErr)?;
    if !tmsu_tag_status.success() {
        return Err(TmsuError::TaggingErr);
    }
    Ok(())
}
