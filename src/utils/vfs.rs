use std::{fs, path::Path};

logger! {
    const LOGGER "File"
}

use serde::{Deserialize, Serialize};

use crate::logger;

pub fn dir(path: &Path) -> crate::Result<()> {
    if !path.exists() {
        LOGGER.info(format!("Directory {path:?} does not exist, creating it"));
        fs::create_dir_all(path)?;
    }
    Ok(())
}

// pub fn read(path: &Path, default_content: &str) -> crate::Result<String> {
//     if !path.exists() {
//         LOGGER.info(format!(
//             "File {path:?} does not exist, creating it with default contents"
//         ));
//         write(path, default_content)?;
//         return Ok(default_content.to_string());
//     }

//     Ok(fs::read_to_string(path)?)
// }

// pub fn read_bytes<'a>(path: &Path, default_content: Vec<u8>) -> crate::Result<Vec<u8>> {
//     if !path.exists() {
//         LOGGER.info(format!(
//             "File {path:?} does not exist, creating it with default contents"
//         ));
//         write_bytes(path, &default_content)?;
//         return Ok(default_content);
//     }

//     Ok(fs::read(path)?)
// }

pub fn read_config<T: Default + Serialize + for<'de> Deserialize<'de>>(
    path: &Path,
) -> crate::Result<T> {
    if !path.exists() {
        LOGGER.info(format!(
            "File {path:?} does not exist, creating it with default contents"
        ));
        let default = T::default();
        write_config(path, &default)?;
        return Ok(default);
    }

    let read = fs::read_to_string(path)?;
    Ok(serde_json::from_str::<T>(&read)?)
}

// pub fn write(path: &Path, content: &str) -> crate::Result<()> {
//     dir(path.parent().ok_or(std::io::Error::new(
//         std::io::ErrorKind::InvalidFilename,
//         "File doesn't have a parent assigned, example: `config/config.json`",
//     ))?)?;
//     fs::write(path, content)?;
//     Ok(())
// }

// pub fn write_bytes(path: &Path, content: &[u8]) -> crate::Result<()> {
//     dir(path.parent().ok_or(std::io::Error::new(
//         std::io::ErrorKind::InvalidFilename,
//         "File doesn't have a parent assigned, example: `config/config.json`",
//     ))?)?;
//     fs::write(path, content)?;
//     Ok(())
// }

pub fn write_config<T: Serialize>(path: &Path, content: &T) -> crate::Result<()> {
    dir(path.parent().ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidFilename,
        "File doesn't have a parent assigned, example: `config/config.json`",
    ))?)?;
    fs::write(path, &serde_json::to_string_pretty(content)?)?;
    Ok(())
}
