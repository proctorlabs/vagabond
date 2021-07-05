mod dhcp;
mod dns;
mod hostapd;
mod http;
mod iwd;
mod process;
mod service_trait;
mod wireguard;

pub use dns::DnsService;
pub use http::HttpServer;
pub use iwd::IwdManager;

pub use process::*;
pub use service_trait::*;

use crate::VagabondConfig;
