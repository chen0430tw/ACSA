// O-Sovereign TUI (Terminal UI using Dioxus TUI)

use dioxus::prelude::*;
use o_sovereign::{create_provider, ACSAConfig, ACSARouter, AgentRole};
use std::sync::Arc;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Launch Dioxus TUI
    dioxus_tui::launch(App);
}

#[component]
fn App() -> Element {
    let mut input_text = use_signal(|| String::new());
    let mut output_text = use_signal(|| String::from("Ready. Enter command and press Enter."));
    let mut status = use_signal(|| String::from("Idle"));

    let on_submit = move |_| {
        let input = input_text();
        status.set("Processing...".to_string());

        spawn(async move {
            match execute_acsa_tui(input.clone()).await {
                Ok(result) => {
                    output_text.set(result);
                    status.set("Completed".to_string());
                }
                Err(e) => {
                    output_text.set(format!("Error: {}", e));
                    status.set("Error".to_string());
                }
            }
        });
    };

    rsx! {
        div {
            style: "width: 100%; height: 100%; flex-direction: column;",

            // Header
            div {
                style: "padding: 1; border-style: double; background-color: blue; color: white;",
                "🤖 O-Sovereign ACSA System (TUI Mode)"
            }

            // Agent Status
            div {
                style: "padding: 1; flex-direction: row;",
                "[MOSS 🧠] [L6 🔬] [Ultron 🛡️ ] [Omega ⚡]"
            }

            // Status
            div {
                style: "padding: 1; background-color: green; color: black;",
                "Status: {status()}"
            }

            // Input
            div {
                style: "padding: 1;",
                "Input: "
                input {
                    value: input_text(),
                    oninput: move |e| input_text.set(e.value())
                }
            }

            // Execute Button
            button {
                onclick: on_submit,
                "Execute ACSA"
            }

            // Output
            div {
                style: "padding: 1; flex: 1; border-style: single;",
                "{output_text()}"
            }

            // Footer
            div {
                style: "padding: 1; background-color: gray;",
                "v0.1.0 | Press Ctrl+C to exit"
            }
        }
    }
}

async fn execute_acsa_tui(input: String) -> anyhow::Result<String> {
    // Use mock mode for TUI
    let moss = create_provider(AgentRole::MOSS, None, true)?;
    let l6 = create_provider(AgentRole::L6, None, true)?;
    let ultron = create_provider(AgentRole::Ultron, None, true)?;
    let omega = create_provider(AgentRole::Omega, None, true)?;

    let config = ACSAConfig::default();
    let router = ACSARouter::new(moss, l6, ultron, omega, config);

    let log = router.execute(input).await?;

    Ok(format!(
        "Success: {} | Time: {}ms | Cost: ${:.4}\n\nOutput:\n{}",
        log.success,
        log.total_time_ms,
        log.total_cost,
        log.final_output.unwrap_or_else(|| "N/A".to_string())
    ))
}
