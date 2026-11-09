use anyhow::Result;
use clap::{Parser, Subcommand};
use lpmr_core::tools::{discover_installed_tools, required_tools};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lpmr", about = "Linux Partitions Manager Rust — web & terminal panel")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start web orchestrator (same as `web`, default 127.0.0.1:8787)
    Start {
        #[arg(long, default_value = "127.0.0.1:8787")]
        bind: String,
    },
    /// Launch the terminal panel (Ratatui, Raven-inspired dock layout)
    Tui,
    /// Launch the web panel (Axum)
    Web {
        #[arg(long, default_value = "127.0.0.1:8787")]
        bind: String,
    },
    /// Print detected system tools
    Doctor,
    /// List required tools and whether they are installed
    Tools,
    /// Run performance benchmark (Rust core vs shell orchestration)
    Bench {
        #[arg(long, default_value_t = 50)]
        iterations: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Start { bind } => {
            let addr: SocketAddr = bind.parse()?;
            let root = workspace_root();
            std::env::set_current_dir(&root)?;
            lpmr_web::serve(addr, root.join("assets")).await?;
        }
        Commands::Tui => lpmr_tui::run_tui()?,
        Commands::Web { bind } => {
            let addr: SocketAddr = bind.parse()?;
            let root = workspace_root();
            std::env::set_current_dir(&root)?;
            lpmr_web::serve(addr, root.join("assets")).await?;
        }
        Commands::Doctor => doctor()?,
        Commands::Tools => tools_list()?,
        Commands::Bench { iterations } => {
            lpmr_bench::run(iterations)?;
        }
    }
    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn doctor() -> Result<()> {
    let installed = discover_installed_tools();
    println!("LPMR doctor — {} tools detected", installed.len());
    for tool in &installed {
        println!("  OK  {} — {}", tool.name, tool.purpose);
    }
    let missing: Vec<_> = required_tools()
        .into_iter()
        .filter(|(name, _)| !installed.iter().any(|t| t.name == *name))
        .collect();
    if missing.is_empty() {
        println!("All recommended tools are installed.");
    } else {
        println!("Missing (optional or platform-specific):");
        for (name, purpose) in missing {
            println!("  - {name}: {purpose}");
        }
        println!("\nInstall on Debian/Ubuntu: sudo ./scripts/install-deps.sh");
    }
    Ok(())
}

fn tools_list() -> Result<()> {
    let json = serde_json::to_string_pretty(&discover_installed_tools())?;
    println!("{json}");
    Ok(())
}

mod lpmr_bench {
    use anyhow::Result;
    use lpmr_core::devices::scan_block_devices;
    use std::process::Command;
    use std::time::Instant;

    pub fn run(iterations: u32) -> Result<()> {
        println!("LPMR benchmark (iterations={iterations})");
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = scan_block_devices()?;
        }
        let rust_elapsed = start.elapsed();

        let start = Instant::now();
        for _ in 0..iterations {
            Command::new("lsblk")
                .args(["--json", "-b", "-O"])
                .output()?;
        }
        let shell_elapsed = start.elapsed();

        let rust_ms = rust_elapsed.as_secs_f64() * 1000.0;
        let shell_ms = shell_elapsed.as_secs_f64() * 1000.0;
        let speedup = shell_ms / rust_ms.max(0.001);

        println!("Rust core scan: {:.2} ms total ({:.3} ms/op)", rust_ms, rust_ms / iterations as f64);
        println!(
            "Shell lsblk loop: {:.2} ms total ({:.3} ms/op)",
            shell_ms,
            shell_ms / iterations as f64
        );
        println!("Speedup (shell/rust): {speedup:.2}x");
        Ok(())
    }
}
