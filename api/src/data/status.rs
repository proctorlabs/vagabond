use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant, Display, Serialize, Deserialize)]
pub enum ServiceState {
    Stopped,
    Running,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub enabled: bool,
    pub state: ServiceState,
    pub detail: HashMap<String, String>,
}
