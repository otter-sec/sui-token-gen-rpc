mod create;
mod health;
mod index;
mod verify;

pub use create::create_handler;
pub use health::health_handler;
pub use index::index;
pub use verify::verify_address_handler;
pub use verify::verify_content_handler;
pub use verify::verify_url_handler;

use crate::TokenServer;

/// Struct representing the shared application state.
///
/// Contains the `TokenServer` instance, which is responsible for processing requests.
pub struct AppState {
    pub server: TokenServer,
}
