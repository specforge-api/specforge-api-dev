//! Command-line argument definitions using Clap.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "specforge",
    author = "SpecForge Team <team@specforge.dev>",
    version = "0.1.0",
    about = "SpecForge Dev: Universal API Studio, Boundary Fuzzer & Smart Mock Engine (Rust)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the interactive Studio Web UI & dynamic mock server
    Serve {
        /// Path or HTTP URL to OpenAPI or OpenAI tool specification
        #[arg(short, long, env = "SPEC_SOURCE", default_value = "/specs/openapi.json")]
        spec: String,

        /// Bind host address
        #[arg(long, env = "SPECFORGE_HOST", default_value = "0.0.0.0")]
        host: String,

        /// Studio Web UI port
        #[arg(short, long, env = "SPECFORGE_PORT", default_value_t = 4000)]
        port: u16,

        /// Target backend URL to run tests against (e.g. http://localhost:8000 or http://api:3000)
        #[arg(short, long, env = "TARGET_URL", default_value = "http://localhost:8000")]
        target_url: String,

        /// Target language export format: 'jest' (Node.js/Express) or 'pytest' (Python)
        #[arg(long, env = "EXPORT_TARGET", default_value = "pytest")]
        export_target: String,

        /// Studio theme: 'dark' or 'light'
        #[arg(long, env = "THEME", default_value = "dark")]
        theme: String,
    },

    /// Headless CI contract testing runner
    Test {
        /// Path or HTTP URL to OpenAPI or OpenAI tool specification
        #[arg(short, long)]
        spec: String,

        /// Target backend base URL
        #[arg(short, long, default_value = "http://localhost:8000")]
        target_url: String,

        /// Fail with exit code 1 if unhandled HTTP 500 server crashes are found
        #[arg(long, default_value_t = true)]
        fail_on_500: bool,

        /// Optional file path to export test suite (e.g. ./tests/contracts.test.ts)
        #[arg(long)]
        export: Option<String>,

        /// Export format ('jest' or 'pytest')
        #[arg(long, default_value = "pytest")]
        format: String,
    },
}
