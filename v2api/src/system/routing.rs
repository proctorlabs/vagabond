use iptables::IPTables;

use super::*;

// Tables
const FILTER: &'static str = "filter";
const NAT: &'static str = "nat";

// Jumps
const DROP: &'static str = "DROP";
const ACCEPT: &'static str = "ACCEPT";
const REJECT: &'static str = "REJECT";
const MASQUERADE: &'static str = "MASQUERADE";

// System chains
const INPUT: &'static str = "INPUT";
const FORWARD: &'static str = "FORWARD";
const OUTPUT: &'static str = "OUTPUT";
const PREROUTING: &'static str = "PREROUTING";
const POSTROUTING: &'static str = "POSTROUTING";

// Vagabond chains
const VAGABOND_INPUT: &'static str = "vagabond-input";
const VAGABOND_FORWARD: &'static str = "vagabond-forward";
const VAGABOND_OUTPUT: &'static str = "vagabond-output";
const VAGABOND_PREROUTING: &'static str = "vagabond-prerouting";
const VAGABOND_POSTROUTING: &'static str = "vagabond-postrouting";

pub fn setup_routes(cfg: &VagabondConfig) -> Result<(), ErrorString> {
    let ipt = iptables::new(false)?;
    setup_filters(&ipt, cfg)?;
    setup_nat(&ipt, cfg)?;
    Ok::<(), ErrorString>(())
}

fn setup_filters(ipt: &IPTables, cfg: &VagabondConfig) -> Result<(), ErrorString> {
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
        ipt.append_unique(FILTER, &oldchain, &format!("-j {}", newchain))?;
    }

    // SETUP SOME GLOBAL RULES
    ipt.append(FILTER, VAGABOND_INPUT, &format!("-i lo -j {}", ACCEPT))?;
    ipt.append(
        FILTER,
        VAGABOND_INPUT,
        &format!("-d 127.0.0.0/8 -j {}", REJECT),
    )?;
    ipt.append(
        FILTER,
        VAGABOND_INPUT,
        &format!("-m state --state ESTABLISHED,RELATED -j {}", ACCEPT),
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
    Ok(())
}

fn setup_nat(ipt: &IPTables, cfg: &VagabondConfig) -> Result<(), ErrorString> {
    // SETUP NAT RULES
    for (oldchain, newchain) in [
        (PREROUTING, VAGABOND_PREROUTING),
        (INPUT, VAGABOND_INPUT),
        (OUTPUT, VAGABOND_OUTPUT),
        (POSTROUTING, VAGABOND_POSTROUTING),
    ] {
        ipt.new_chain(NAT, &newchain).unwrap_or_default();
        ipt.flush_chain(NAT, &newchain).unwrap_or_default();
        ipt.append_unique(NAT, &oldchain, &format!("-j {}", newchain))?;
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
    Ok(())
}

#[derive(Display, Debug)]
pub struct ErrorString(String);
impl From<Box<dyn std::error::Error>> for ErrorString {
    fn from(_: Box<dyn std::error::Error>) -> Self {
        todo!()
    }
}
