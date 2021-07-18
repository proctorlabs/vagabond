use anyhow::Result;
use std::ffi::OsStr;

pub async fn run_command<S: AsRef<OsStr>, I: IntoIterator<Item = S>>(
    cmd: S,
    args: I,
) -> Result<()> {
    let cmd_process = tokio::process::Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()?;

    let output = cmd_process.wait_with_output().await?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Command failed with error {}\n{}\n{}",
            output.status,
            std::str::from_utf8(&output.stdout)?,
            std::str::from_utf8(&output.stderr)?,
        ))
    }
}
