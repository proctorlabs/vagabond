use anyhow::Result;
use sysctl::*;

pub const CTL_IP4_FORWARD: &'static str = "net/ipv4/ip_forward"; //1
pub const CTL_IP6_DEFAULT_FORWARDING: &'static str = "net/ipv6/conf/default/forwarding"; //1
pub const CTL_IP6_ALL_FORWARDING: &'static str = "net/ipv6/conf/all/forwarding"; //1

pub const CTL_ICMP_IGNORE_ECHO_BROADCAST: &'static str = "net/ipv4/icmp_echo_ignore_broadcasts"; //1
pub const CTL_ICMP_IGNORE_BOGUS_ERRORS: &'static str = "net/ipv4/icmp_ignore_bogus_error_responses"; //1
pub const CTL_ICMP_ECHO_IGNORE_ALL: &'static str = "net/ipv4/icmp_echo_ignore_all"; //0

pub const CTL_IP4_LOG_MARTIANS: &'static str = "net/ipv4/conf/all/log_martians"; //0
pub const CTL_IP6_LOG_MARTIANS: &'static str = "net/ipv4/conf/default/log_martians"; //0

pub fn set_sysctls() -> Result<()> {
    Ctl::new(CTL_IP4_FORWARD)?.set_value_string("1")?;
    Ctl::new(CTL_IP6_DEFAULT_FORWARDING)?.set_value_string("1")?;
    Ctl::new(CTL_IP6_ALL_FORWARDING)?.set_value_string("1")?;
    Ctl::new(CTL_ICMP_IGNORE_ECHO_BROADCAST)?.set_value_string("1")?;
    Ctl::new(CTL_ICMP_IGNORE_BOGUS_ERRORS)?.set_value_string("1")?;
    Ctl::new(CTL_ICMP_ECHO_IGNORE_ALL)?.set_value_string("0")?;
    Ctl::new(CTL_IP4_LOG_MARTIANS)?.set_value_string("0")?;
    Ctl::new(CTL_IP6_LOG_MARTIANS)?.set_value_string("0")?;
    Ok(())
}
