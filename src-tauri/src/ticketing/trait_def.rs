use super::types::*;

/// Trait defining the interface for ticketing integrations
///
/// Implementations must support:
/// - Authentication with API credentials
/// - Creating tickets with title, description, and attachments
/// - Checking connection status
pub trait TicketingIntegration: Send + Sync {
    /// Authenticate with the ticketing service
    ///
    /// # Arguments
    /// * `credentials` - API credentials for authentication
    ///
    /// # Returns
    /// * `Ok(())` if authentication succeeds
    /// * `Err(TicketingError)` if authentication fails
    fn authenticate(&self, credentials: &TicketingCredentials) -> TicketingResult<()>;

    /// Create a ticket in the ticketing system
    ///
    /// # Arguments
    /// * `request` - Details of the ticket to create
    ///
    /// # Returns
    /// * `Ok(CreateTicketResponse)` with ticket details
    /// * `Err(TicketingError)` if creation fails
    fn create_ticket(&self, request: &CreateTicketRequest) -> TicketingResult<CreateTicketResponse>;

    /// Check if the connection to the ticketing service is working
    ///
    /// # Returns
    /// * `Ok(ConnectionStatus)` with connection details
    /// * `Err(TicketingError)` if check fails
    fn check_connection(&self) -> TicketingResult<ConnectionStatus>;

    /// Get the name of this integration (e.g., "Linear", "Jira")
    #[allow(dead_code)]
    fn name(&self) -> &str;
}
