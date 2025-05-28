use color_eyre::eyre::{Context, eyre};

pub fn username_to_id(username: &str) -> color_eyre::Result<u32> {
    let output = std::process::Command::new("id")
        .arg("-u")
        .arg(username)
        .output()
        .wrap_err("Failed to execute id bin")?;

    if !output.status.success() {
        return Err(eyre!("id command failed"));
    }

    let id_str = std::str::from_utf8(&output.stdout).wrap_err("Failed to parse id output")?;
    id_str.trim().parse().wrap_err("Failed to parse user ID")
}

pub fn groupname_to_id(groupname: &str) -> color_eyre::Result<u32> {
    let output = std::process::Command::new("id")
        .arg("-g")
        .arg(groupname)
        .output()
        .wrap_err("Failed to execute id bin")?;

    if !output.status.success() {
        return Err(eyre!("id command failed"));
    }

    let id_str = std::str::from_utf8(&output.stdout).wrap_err("Failed to parse id output")?;
    id_str.trim().parse().wrap_err("Failed to parse group ID")
}
