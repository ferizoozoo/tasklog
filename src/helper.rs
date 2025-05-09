use std::env;

pub fn get_home_directory() -> Result<String, String> {
    if let Ok(path) = env::var("HOME") {
        return Ok(path);
    }

    if let Ok(path) = env::var("USERPROFILE") {
        return Ok(path);
    }

    if let (Ok(drive), Ok(path)) = (env::var("HOMEDRIVE"), env::var("HOMEPATH")) {
        return Ok(format!("{}{}", drive, path));
    }

    return Err("Could not get the home directory".to_string());
}
