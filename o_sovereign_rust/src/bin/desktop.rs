// O-Sovereign Desktop UI (Dioxus)

use dioxus::prelude::*;
use o_sovereign::{create_provider, ACSAConfig, ACSARouter, AgentRole};
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Launch Dioxus 0.7 desktop app
    // Note: 0.7 uses simplified launch API
    launch(App);
}

#[component]
fn App() -> Element {
    // State
    let mut input_text = use_signal(|| String::new());
    let mut output_text = use_signal(|| String::from("ACSA Output will appear here..."));
    let mut is_processing = use_signal(|| false);
    let mut use_mock = use_signal(|| true);
    let mut risk_threshold = use_signal(|| 70u8);

    // Handler for execute button
    let on_execute = move |_| {
        let input = input_text();
        let mock = use_mock();
        let threshold = risk_threshold();

        is_processing.set(true);
        output_text.set(String::from("Processing..."));

        spawn(async move {
            match execute_acsa(input.clone(), mock, threshold).await {
                Ok(result) => {
                    output_text.set(result);
                }
                Err(e) => {
                    output_text.set(format!("Error: {}", e));
                }
            }
            is_processing.set(false);
        });
    };

    rsx! {
        style { {include_str!("../ui/styles.css")} }

        div { class: "container",
            // Header
            div { class: "header",
                h1 { "🤖 O-Sovereign ACSA System" }
                p { class: "subtitle",
                    "Adversarially-Constrained Sycophantic Agent"
                }
            }

            // Agent Status Bar
            div { class: "agent-status-bar",
                AgentStatusBadge { role: AgentRole::MOSS, status: "Idle" }
                AgentStatusBadge { role: AgentRole::L6, status: "Idle" }
                AgentStatusBadge { role: AgentRole::Ultron, status: "Idle" }
                AgentStatusBadge { role: AgentRole::Omega, status: "Idle" }
            }

            // Settings Panel
            div { class: "settings-panel",
                label {
                    input {
                        r#type: "checkbox",
                        checked: use_mock(),
                        onchange: move |e| use_mock.set(e.value().parse().unwrap_or(false))
                    }
                    " Use Mock Mode (no API keys required)"
                }

                label {
                    "Risk Threshold: {risk_threshold()}"
                    input {
                        r#type: "range",
                        min: 0,
                        max: 100,
                        value: risk_threshold(),
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u8>() {
                                risk_threshold.set(val);
                            }
                        }
                    }
                }
            }

            // Input Area
            div { class: "input-area",
                h3 { "User Input:" }
                textarea {
                    class: "input-box",
                    placeholder: "Enter your request here...",
                    value: input_text(),
                    oninput: move |e| input_text.set(e.value())
                }

                button {
                    class: "execute-button",
                    disabled: is_processing(),
                    onclick: on_execute,
                    if is_processing() {
                        "⏳ Processing..."
                    } else {
                        "🚀 Execute ACSA"
                    }
                }
            }

            // Output Area
            div { class: "output-area",
                h3 { "ACSA Output:" }
                div { class: "output-box",
                    dangerous_inner_html: format_output(output_text())
                }
            }

            // Footer
            div { class: "footer",
                "O-Sovereign v0.1.0 | ACSA Architecture | Powered by Dioxus & Rust"
            }
        }
    }
}

#[component]
fn AgentStatusBadge(role: AgentRole, status: &'static str) -> Element {
    rsx! {
        div { class: "agent-badge",
            span { class: "agent-icon", "{role.emoji()}" }
            div { class: "agent-info",
                div { class: "agent-name", "{role.as_str()}" }
                div { class: "agent-status status-{status.to_lowercase()}",
                    "{status}"
                }
            }
        }
    }
}

fn format_output(text: String) -> String {
    text.replace("\n", "<br>")
        .replace("  ", "&nbsp;&nbsp;")
}

async fn execute_acsa(input: String, use_mock: bool, risk_threshold: u8) -> anyhow::Result<String> {
    // Get API keys from environment
    let openai_key = if !use_mock {
        std::env::var("OPENAI_API_KEY").ok()
    } else {
        None
    };

    // Create providers
    let moss = create_provider(AgentRole::MOSS, openai_key.clone(), use_mock)?;
    let l6 = create_provider(AgentRole::L6, None, use_mock)?;
    let ultron = create_provider(AgentRole::Ultron, None, use_mock)?;
    let omega = create_provider(AgentRole::Omega, None, use_mock)?;

    // Create router
    let config = ACSAConfig {
        max_iterations: 3,
        risk_threshold,
        enable_l6: true,
        enable_streaming: false,
    };

    let router = ACSARouter::new(moss, l6, ultron, omega, config);

    // Execute
    let log = router.execute(input).await?;

    // Format result
    let mut result = String::new();
    result.push_str(&format!("🎯 Success: {}\n", log.success));
    result.push_str(&format!("⏱️  Time: {} ms\n", log.total_time_ms));
    result.push_str(&format!("💰 Cost: ${:.4}\n", log.total_cost));
    result.push_str(&format!("🔁 Iterations: {}\n\n", log.iterations));

    if let Some(audit) = &log.audit_result {
        result.push_str(&format!("🛡️  Risk Score: {}/100\n", audit.risk_score));
        result.push_str(&format!("✅ Safe: {}\n\n", audit.is_safe));
    }

    result.push_str("📝 Final Output:\n");
    result.push_str(&format!(
        "{}",
        log.final_output.unwrap_or_else(|| "N/A".to_string())
    ));

    Ok(result)
}
