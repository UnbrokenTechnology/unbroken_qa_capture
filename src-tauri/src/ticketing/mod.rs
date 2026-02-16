/// Ticketing integration module for creating issues in external systems
///
/// Supports pluggable integrations via the TicketingIntegration trait.
/// Currently implements Linear, with planned support for Jira and GitHub.
mod types;
mod trait_def;
mod linear;

pub use types::*;
pub use trait_def::TicketingIntegration;
pub use linear::LinearIntegration;

#[cfg(test)]
mod tests;
