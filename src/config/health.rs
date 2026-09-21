#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Clone, Debug, ClapArgs)]
pub struct HealthArgs {
    /// Optional standard health service name; empty checks the server overall.
    #[arg(default_value = "")]
    pub service: String,
}
