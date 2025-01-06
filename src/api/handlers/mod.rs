mod create;
mod health;
mod verify;
mod index;

pub use create::create_handler;
pub use health::health_handler;
pub use verify::verify_content_handler;
pub use verify::verify_url_handler;
pub use index::index;

use crate::TokenServer;


/// Struct representing the shared application state.
///
/// Contains the `TokenServer` instance, which is responsible for processing requests.
pub struct AppState {
    pub server: TokenServer,
}
