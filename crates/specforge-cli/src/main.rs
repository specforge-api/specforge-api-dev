//! SpecForge CLI main entrypoint.

use clap::Parser;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
use config::{Cli, Commands};

use specforge_core::models::ApiSpec;
use specforge_core::openapi::OpenAPIParser;
use specforge_core::openai::OpenAiToolParser;
use specforge_server::{create_router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            spec,
            host,
            port,
            target_url,
            export_target,
            theme,
        } => {
            println!("\n⚡ \x1b[1;36mSpecForge Dev Engine (Rust)\x1b[0m v0.1.0");
            println!("📖 Loading Specification: \x1b[33m{}\x1b[0m", spec);

            let raw_value = load_spec(&spec).await?;
            let api_spec = parse_spec_auto(&raw_value)?;

            println!(
                "✅ Parsed \x1b[32m{} operations\x1b[0m for: \x1b[1m{}\x1b[0m (v{})",
                api_spec.operations.len(),
                api_spec.title,
                api_spec.version
            );
            println!("🎯 Target Backend: \x1b[34m{}\x1b[0m", target_url);
            println!("🛠️  Export Format:  \x1b[35m{}\x1b[0m (Node.js/Jest or Python/Pytest)", export_target);

            let service_name = std::env::var("SERVICE_NAME").ok()
                .or_else(|| std::env::var("PROJECT_NAME").ok())
                .filter(|s| !s.trim().is_empty());

            if let Some(ref svc) = service_name {
                println!("🏷️  Service Scope:  \x1b[36m{}\x1b[0m", svc);
            }

            let app_state = Arc::new(AppState {
                spec: RwLock::new(api_spec),
                spec_source: spec.clone(),
                target_url,
                export_target,
                theme,
                service_name,
                version: std::sync::atomic::AtomicU64::new(1),
            });

            // Spawn background live-reload watcher
            let watcher_state = app_state.clone();
            let watcher_source = spec.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
                let mut last_content: Option<String> = None;
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(3))
                    .build()
                    .unwrap_or_default();

                loop {
                    interval.tick().await;

                    let fetch_res = if watcher_source.starts_with("http://") || watcher_source.starts_with("https://") {
                        match client.get(&watcher_source).send().await {
                            Ok(resp) if resp.status().is_success() => resp.text().await.ok(),
                            _ => None,
                        }
                    } else {
                        tokio::fs::read_to_string(&watcher_source).await.ok()
                    };

                    if let Some(content) = fetch_res {
                        if let Some(ref prev) = last_content {
                            if *prev != content {
                                let maybe_new_spec: Option<ApiSpec> = {
                                    let parsed_val: Option<serde_json::Value> = serde_json::from_str(&content)
                                        .or_else(|_| serde_yaml::from_str(&content))
                                        .ok();

                                    parsed_val.and_then(|raw_val| parse_spec_auto(&raw_val).ok())
                                };

                                if let Some(new_spec) = maybe_new_spec {
                                    let op_count = new_spec.operations.len();
                                    let title = new_spec.title.clone();
                                    {
                                        let mut guard = watcher_state.spec.write().await;
                                        *guard = new_spec;
                                    }
                                    watcher_state.version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                    println!("⚡ \x1b[1;33m[SpecForge Auto-Reload]\x1b[0m Detected schema update! Hot-reloaded \x1b[32m{} operations\x1b[0m for: \x1b[1m{}\x1b[0m", op_count, title);
                                }
                            }
                        }
                        last_content = Some(content);
                    }
                }
            });

            let router = create_router(app_state);
            let addr = format!("{}:{}", host, port);
            let listener = tokio::net::TcpListener::bind(&addr).await?;

            println!("\n🚀 \x1b[1;32mSpecForge Studio Web UI:\x1b[0m http://localhost:{}", port);
            println!("🎭 \x1b[1;35mDynamic Mock Server:    \x1b[0m http://localhost:{}/mock/*", port);
            println!("============================================================");

            axum::serve(listener, router).await?;
        }

        Commands::Test {
            spec,
            target_url,
            fail_on_500,
            export,
            format,
        } => {
            println!("\n⚡ \x1b[1;36mSpecForge Headless Contract Fuzzer\x1b[0m");
            let raw_value = load_spec(&spec).await?;
            let api_spec = parse_spec_auto(&raw_value)?;

            println!(
                "Loaded \x1b[32m{} operations\x1b[0m against \x1b[33m{}\x1b[0m",
                api_spec.operations.len(),
                target_url
            );

            // Execute headless tests
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()?;

            let mut all_test_cases = Vec::new();
            for op in &api_spec.operations {
                let plan = specforge_fuzzer::planner::TestPlanner::plan_operation_tests(op);
                all_test_cases.extend(plan.test_cases);
            }

            println!("Synthesized \x1b[1;35m{}\x1b[0m boundary tests.\n", all_test_cases.len());

            let mut pass_count = 0;
            let mut crash_count = 0;
            let mut leak_count = 0;

            for tc in &all_test_cases {
                let url = format!("{}{}", target_url.trim_end_matches('/'), tc.endpoint);
                let start = std::time::Instant::now();
                let method = match tc.method.to_uppercase().as_str() {
                    "POST" => reqwest::Method::POST,
                    "PUT" => reqwest::Method::PUT,
                    "DELETE" => reqwest::Method::DELETE,
                    "PATCH" => reqwest::Method::PATCH,
                    _ => reqwest::Method::GET,
                };

                let mut req = client.request(method, &url);
                if let Some(ref p) = tc.payload {
                    req = req.json(p);
                }

                let actual_status = match req.send().await {
                    Ok(r) => r.status().as_u16(),
                    Err(_) => 599,
                };

                let latency = start.elapsed().as_secs_f64() * 1000.0;
                let result = specforge_fuzzer::invariants::evaluate_test_result(tc, actual_status, "", latency);

                match result.status {
                    specforge_fuzzer::invariants::TestStatus::Pass => {
                        pass_count += 1;
                        println!("  \x1b[32m[PASS]\x1b[0m {} {} ({}ms)", tc.method.to_uppercase(), tc.endpoint, latency as u64);
                    }
                    specforge_fuzzer::invariants::TestStatus::Crash500 => {
                        crash_count += 1;
                        println!("  \x1b[1;31m[CRASH 500]\x1b[0m {} {} - {}", tc.method.to_uppercase(), tc.endpoint, result.message);
                    }
                    specforge_fuzzer::invariants::TestStatus::WarnLeak => {
                        leak_count += 1;
                        println!("  \x1b[33m[LEAK 200]\x1b[0m {} {} - {}", tc.method.to_uppercase(), tc.endpoint, result.message);
                    }
                    _ => {
                        println!("  \x1b[31m[{}]\x1b[0m {} {}", actual_status, tc.method.to_uppercase(), tc.endpoint);
                    }
                }
            }

            println!("\n📊 Final Summary: Total: {}, Passed: {}, Leaks: {}, Crashes: {}",
                all_test_cases.len(), pass_count, leak_count, crash_count);

            if let Some(out_path) = export {
                let code = match format.to_lowercase().as_str() {
                    "jest" | "vitest" => specforge_exporters::JestExporter::export(&all_test_cases, &target_url),
                    _ => specforge_exporters::PytestExporter::export(&all_test_cases, &target_url),
                };
                std::fs::write(&out_path, code)?;
                println!("🐍 Exported test suite to: {}", out_path);
            }

            if fail_on_500 && crash_count > 0 {
                eprintln!("\x1b[1;31mFAILED: {} unhandled 500 crashes detected!\x1b[0m", crash_count);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

async fn load_spec(target: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let content = if target.starts_with("http://") || target.starts_with("https://") {
        reqwest::get(target).await?.text().await?
    } else {
        std::fs::read_to_string(target)?
    };

    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
        Ok(json_val)
    } else {
        let yaml_val: serde_json::Value = serde_yaml::from_str(&content)?;
        Ok(yaml_val)
    }
}

fn parse_spec_auto(raw: &serde_json::Value) -> Result<ApiSpec, Box<dyn std::error::Error + Send + Sync>> {
    // Check if it's an OpenAI tools definition array
    if raw.is_array() || raw.get("tools").is_some() || raw.get("functions").is_some() {
        if let Ok(spec) = OpenAiToolParser::parse(raw) {
            return Ok(spec);
        }
    }

    // Standard OpenAPI 3.0 / 3.1 & Swagger 2.0 parser
    let parser = OpenAPIParser::new(raw.clone());
    let spec = parser.parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(spec)
}

