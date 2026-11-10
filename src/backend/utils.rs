use std::{fs::{self}, path::PathBuf};
use anyhow::anyhow;

pub fn app_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe.parent()
        .ok_or_else( || anyhow!("Failed to determine app's directory\n")
    )?;  
    Ok(dir.to_path_buf())
}