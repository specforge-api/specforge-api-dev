//! SpecForge Server: Axum REST API, embedded UI server, and dynamic mock router.

pub mod app;
pub mod embedded;
pub mod routes {
    pub mod docs;
    pub mod export;
    pub mod mock;
    pub mod plan;
    pub mod proxy;
    pub mod run;
}

pub use app::{create_router, AppState};

