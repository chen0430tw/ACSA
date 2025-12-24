// O-Sovereign CLI
// Command-line interface for ACSA system

use clap::{Parser, Subcommand};
use o_sovereign::{create_provider, ACSAConfig, ACSARouter, AgentRole};

#[derive(Parser)]
#[command(name = "o-sovereign")]
#[command(about = "ACSA (对抗约束型盲从代理) CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute ACSA chain
    Execute {
        /// Input text
        #[arg(short, long)]
        input: String,

        /// Use mock mode (no API keys)
        #[arg(short, long)]
        mock: bool,

        /// Risk threshold (0-100)
        #[arg(short, long, default_value_t = 70)]
        threshold: u8,
    },

    /// Show version
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Execute { input, mock, threshold } => {
            execute_cli(input, mock, threshold).await?;
        }
        Commands::Version => {
            println!("O-Sovereign v0.1.0 (Rust Edition)");
        }
    }

    Ok(())
}

async fn execute_cli(input: String, use_mock: bool, risk_threshold: u8) -> anyhow::Result<()> {
    println!("\n{}", "=".repeat(80));
    println!("🚀 O-Sovereign ACSA CLI");
    println!("{}", "=".repeat(80));

    let openai_key = if !use_mock { std::env::var("OPENAI_API_KEY").ok() } else { None };

    let moss = create_provider(AgentRole::MOSS, openai_key, use_mock)?;
    let l6 = create_provider(AgentRole::L6, None, use_mock)?;
    let ultron = create_provider(AgentRole::Ultron, None, use_mock)?;
    let omega = create_provider(AgentRole::Omega, None, use_mock)?;

    let config = ACSAConfig {
        max_iterations: 3,
        risk_threshold,
        enable_l6: true,
        enable_streaming: false,
    };

    let router = ACSARouter::new(moss, l6, ultron, omega, config);
    let log = router.execute(input).await?;

    println!("\n📊 Results:");
    println!("✅ Success: {}", log.success);
    println!("⏱️  Time: {} ms", log.total_time_ms);
    println!("💰 Cost: ${:.4}", log.total_cost);
    println!("\n📝 Output:\n{}", log.final_output.unwrap_or_else(|| "N/A".to_string()));

    Ok(())
}
