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
    let mut show_settings = use_signal(|| false);
    let mut language = use_signal(|| "zh-CN".to_string());

    // API key states (not persisted, just for UI demo)
    let mut api_key_openai = use_signal(|| String::new());
    let mut api_key_deepseek = use_signal(|| String::new());

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

                // Language Switcher
                div { class: "language-switcher",
                    select {
                        value: language(),
                        onchange: move |e| language.set(e.value()),
                        option { value: "zh-CN", "🇨🇳 简体中文" }
                        option { value: "en-US", "🇺🇸 English" }
                        option { value: "ja-JP", "🇯🇵 日本語" }
                        option { value: "ko-KR", "🇰🇷 한국어" }
                    }
                }
            }

            // Agent Status Bar
            div { class: "agent-status-bar",
                AgentStatusBadge { role: AgentRole::MOSS, status: "Idle" }
                AgentStatusBadge { role: AgentRole::L6, status: "Idle" }
                AgentStatusBadge { role: AgentRole::Ultron, status: "Idle" }
                AgentStatusBadge { role: AgentRole::Omega, status: "Idle" }
            }

            // Mock Mode Warning Banner
            if use_mock() {
                div { class: "mock-warning-banner",
                    "⚠️ MOCK MODE ACTIVE - 这不是真实AI！响应是硬编码的测试数据（只回显输入）"
                    br {}
                    "💡 要使用真实AI，请在下方配置API密钥并取消勾选Mock模式"
                }
            }

            // Settings Panel with Toggle
            div { class: "settings-section",
                button {
                    class: "settings-toggle-btn",
                    onclick: move |_| show_settings.set(!show_settings()),
                    if show_settings() { "▼ 隐藏设置" } else { "▶ 显示设置" }
                }

                if show_settings() {
                    div { class: "settings-panel-expanded",
                        // API Configuration
                        div { class: "api-config-section",
                            h4 { "🔑 API 配置" }
                            p { class: "config-hint",
                                "注意：当前UI不保存密钥。生产环境请使用 .env 文件配置！"
                            }

                            label {
                                "OpenAI API Key:"
                                input {
                                    r#type: "password",
                                    placeholder: "sk-...",
                                    value: api_key_openai(),
                                    oninput: move |e| api_key_openai.set(e.value())
                                }
                            }

                            label {
                                "DeepSeek API Key (推荐国内用户):"
                                input {
                                    r#type: "password",
                                    placeholder: "sk-...",
                                    value: api_key_deepseek(),
                                    oninput: move |e| api_key_deepseek.set(e.value())
                                }
                            }

                            a {
                                class: "help-link",
                                href: "https://github.com/chen0430tw/ACSA/blob/main/docs/guides/GETTING_STARTED.md#第三步配置真实-api可选",
                                target: "_blank",
                                "📘 查看完整API配置指南"
                            }
                        }

                        // Runtime Settings
                        div { class: "runtime-settings",
                            h4 { "⚙️ 运行时设置" }

                            label { class: "checkbox-label",
                                input {
                                    r#type: "checkbox",
                                    checked: use_mock(),
                                    onchange: move |e| use_mock.set(e.value().parse().unwrap_or(false))
                                }
                                span { class: "checkbox-text",
                                    " Use Mock Mode "
                                    span { class: "badge-mock", "免费测试" }
                                }
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
                                span { class: "threshold-hint",
                                    if risk_threshold() < 30 {
                                        " (宽松 - 允许更多操作)"
                                    } else if risk_threshold() > 70 {
                                        " (严格 - Jarvis更容易阻止)"
                                    } else {
                                        " (平衡)"
                                    }
                                }
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
