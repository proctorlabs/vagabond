use anyhow::Result;
use serde::Deserialize;
use std::ffi::OsStr;

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct IPRule {
    pub priority: usize,
    pub src: RouteType,
    pub table: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub enum RouteType {
    All,
    Unicast,
    Unreachable,
    Blackhole,
    Prohibit,
    Local,
    Broadcast,
    Throw,
    Nat,
    Anycast,
    Multicast,
}

impl Default for RouteType {
    fn default() -> Self {
        Self::All
    }
}

#[allow(dead_code)]
impl IPRule {
    pub async fn get_current() -> Result<Vec<Self>> {
        let output = ip_command(["-j", "rule"]).await?;
        Ok(serde_json::from_str(output.as_str())?)
    }
}

pub async fn ip_command<S: AsRef<OsStr>, I: IntoIterator<Item = S>>(args: I) -> Result<String> {
    let cmd_process = tokio::process::Command::new("ip")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()?;

    let output = cmd_process.wait_with_output().await?;
    if output.status.success() {
        Ok(std::str::from_utf8(&output.stdout)?.to_string())
    } else {
        Err(anyhow::anyhow!(
            "Command failed with error {}\n{}\n{}",
            output.status,
            std::str::from_utf8(&output.stdout)?,
            std::str::from_utf8(&output.stderr)?,
        ))
    }
}
