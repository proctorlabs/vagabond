use crate::config::VagabondConfig;
use anyhow::{anyhow, Result};
use nix::unistd;
use std::sync::Arc;
use sysctl::*;
use tokio::task::spawn_blocking;

#[derive(Debug)]
pub struct SystemInfo {
    config: VagabondConfig,
    pub is_root: bool,
}

#[derive(Debug, Clone, Deref)]
pub struct SystemManager(Arc<SystemInfo>);

const FILTER: &'static str = "filter";
const NAT: &'static str = "nat";

const DROP: &'static str = "DROP";
const ACCEPT: &'static str = "ACCEPT";
const MASQUERADE: &'static str = "MASQUERADE";

const INPUT: &'static str = "INPUT";
const FORWARD: &'static str = "FORWARD";
const OUTPUT: &'static str = "OUTPUT";
const PREROUTING: &'static str = "PREROUTING";
const POSTROUTING: &'static str = "POSTROUTING";

const VAGABOND_INPUT: &'static str = "vagabond-input";
const VAGABOND_FORWARD: &'static str = "vagabond-forward";
const VAGABOND_OUTPUT: &'static str = "vagabond-output";
const VAGABOND_PREROUTING: &'static str = "vagabond-prerouting";
const VAGABOND_POSTROUTING: &'static str = "vagabond-postrouting";

impl SystemManager {
    pub const CTL_IP4_FORWARD: &'static str = "net/ipv4/ip_forward"; //1
    pub const CTL_IP6_DEFAULT_FORWARDING: &'static str = "net/ipv6/conf/default/forwarding"; //1
    pub const CTL_IP6_ALL_FORWARDING: &'static str = "net/ipv6/conf/all/forwarding"; //1

    pub const CTL_ICMP_IGNORE_ECHO_BROADCAST: &'static str = "net/ipv4/icmp_echo_ignore_broadcasts"; //1
    pub const CTL_ICMP_IGNORE_BOGUS_ERRORS: &'static str =
        "net/ipv4/icmp_ignore_bogus_error_responses"; //1
    pub const CTL_ICMP_ECHO_IGNORE_ALL: &'static str = "net/ipv4/icmp_echo_ignore_all"; //0

    pub const CTL_IP4_LOG_MARTIANS: &'static str = "net/ipv4/conf/all/log_martians"; //0
    pub const CTL_IP6_LOG_MARTIANS: &'static str = "net/ipv4/conf/default/log_martians"; //0

    pub fn new(config: &VagabondConfig) -> Self {
        let is_root = unistd::geteuid().is_root();
        if !is_root {
            warn!(
                "NOTICE: Some functionality will be disabled as this app is not running as root!"
            );
        }
        SystemManager(Arc::new(SystemInfo {
            config: config.clone(),
            is_root,
        }))
    }

    pub fn setup_sysctl(&self) -> Result<()> {
        if self.is_root {
            Ctl::new(Self::CTL_IP4_FORWARD)?.set_value_string("1")?;
            Ctl::new(Self::CTL_IP6_DEFAULT_FORWARDING)?.set_value_string("1")?;
            Ctl::new(Self::CTL_IP6_ALL_FORWARDING)?.set_value_string("1")?;
            Ctl::new(Self::CTL_ICMP_IGNORE_ECHO_BROADCAST)?.set_value_string("1")?;
            Ctl::new(Self::CTL_ICMP_IGNORE_BOGUS_ERRORS)?.set_value_string("1")?;
            Ctl::new(Self::CTL_ICMP_ECHO_IGNORE_ALL)?.set_value_string("0")?;
            Ctl::new(Self::CTL_IP4_LOG_MARTIANS)?.set_value_string("0")?;
            Ctl::new(Self::CTL_IP6_LOG_MARTIANS)?.set_value_string("0")?;
        } else {
            warn!("Cannot set sysctl when not root!");
        }
        Ok(())
    }

    pub async fn setup_iptables(&self) -> Result<()> {
        if self.is_root {
            let cfg = self.config.clone();
            spawn_blocking(move || {
                let ipt = iptables::new(false)?;
                // POLICIES
                ipt.set_policy(FILTER, INPUT, DROP)?;
                ipt.set_policy(FILTER, OUTPUT, ACCEPT)?;
                ipt.set_policy(FILTER, FORWARD, ACCEPT)?;

                // LINK OUR CHAINS
                for (oldchain, newchain) in [
                    (INPUT, VAGABOND_INPUT),
                    (FORWARD, VAGABOND_FORWARD),
                    (OUTPUT, VAGABOND_OUTPUT),
                ] {
                    ipt.new_chain(FILTER, &newchain).unwrap_or_default();
                    ipt.flush_chain(FILTER, &newchain).unwrap_or_default();
                    if !ipt.exists(FILTER, &oldchain, &format!("-j {}", newchain))? {
                        ipt.append(FILTER, &oldchain, &format!("-j {}", newchain))?;
                    }
                }

                // SETUP SOME GLOBAL RULES
                ipt.append(FILTER, VAGABOND_INPUT, "-i lo -j ACCEPT")?;
                ipt.append(FILTER, VAGABOND_INPUT, "-d 127.0.0.0/8 -j REJECT")?;
                ipt.append(
                    FILTER,
                    VAGABOND_INPUT,
                    "-m state --state ESTABLISHED,RELATED -j ACCEPT",
                )?;

                // SETUP INTERNAL INTERFACE BASE RULES
                for (enabled, iface) in [
                    (cfg.network.lan.enabled, &cfg.network.lan.interface),
                    (cfg.network.wlan.enabled, &cfg.network.wlan.interface),
                    (cfg.wireguard.enabled, &cfg.wireguard.interface),
                ] {
                    if enabled {
                        ipt.append(
                            FILTER,
                            VAGABOND_INPUT,
                            &format!("-i {} -j {}", iface, ACCEPT),
                        )?;
                        ipt.append(
                            FILTER,
                            VAGABOND_FORWARD,
                            &format!("-i {} -j {}", iface, ACCEPT),
                        )?;
                        ipt.append(
                            FILTER,
                            VAGABOND_FORWARD,
                            &format!("-o {} -j {}", iface, ACCEPT),
                        )?;
                    }
                }

                // SETUP NAT RULES
                for (oldchain, newchain) in [
                    (PREROUTING, VAGABOND_PREROUTING),
                    (INPUT, VAGABOND_INPUT),
                    (OUTPUT, VAGABOND_OUTPUT),
                    (POSTROUTING, VAGABOND_POSTROUTING),
                ] {
                    ipt.new_chain(NAT, &newchain).unwrap_or_default();
                    ipt.flush_chain(NAT, &newchain).unwrap_or_default();
                    if !ipt.exists(NAT, &oldchain, &format!("-j {}", newchain))? {
                        ipt.append(NAT, &oldchain, &format!("-j {}", newchain))?;
                    }
                }

                for iface in cfg.external_interfaces() {
                    ipt.append(
                        NAT,
                        VAGABOND_POSTROUTING,
                        &format!("-o {} -j {}", iface, MASQUERADE),
                    )?;
                }

                if cfg.wireguard.enabled {
                    ipt.append(
                        NAT,
                        VAGABOND_POSTROUTING,
                        &format!("-o {} -j {}", cfg.wireguard.interface, MASQUERADE),
                    )?;
                }

                Ok::<(), ErrorString>(())
            })
            .await?
            .map_err(|e| anyhow!(format!("{}", e)))?;
            Ok(())
        } else {
            warn!("IPTables cannot be setup as a non-root user!");
            Ok(())
        }
    }
}

#[derive(Display, Debug)]
pub struct ErrorString(String);
impl From<Box<dyn std::error::Error>> for ErrorString {
    fn from(_: Box<dyn std::error::Error>) -> Self {
        todo!()
    }
}
