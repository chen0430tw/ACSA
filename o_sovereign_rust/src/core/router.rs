// O-Sovereign ACSA Router
// 对抗性路由循环核心逻辑

use super::providers::ModelProvider;
use super::types::{
    ACSAConfig, ACSAExecutionLog, AgentResponse, AgentRole, AuditResult,
};
use anyhow::{anyhow, Result};
use regex::Regex;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// ACSA Router
pub struct ACSARouter {
    moss: Arc<dyn ModelProvider>,
    l6: Arc<dyn ModelProvider>,
    ultron: Arc<dyn ModelProvider>,
    omega: Arc<dyn ModelProvider>,
    config: ACSAConfig,
    execution_logs: Arc<tokio::sync::Mutex<Vec<ACSAExecutionLog>>>,
}

impl ACSARouter {
    pub fn new(
        moss: Arc<dyn ModelProvider>,
        l6: Arc<dyn ModelProvider>,
        ultron: Arc<dyn ModelProvider>,
        omega: Arc<dyn ModelProvider>,
        config: ACSAConfig,
    ) -> Self {
        Self {
            moss,
            l6,
            ultron,
            omega,
            config,
            execution_logs: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Execute ACSA chain
    pub async fn execute(&self, user_input: String) -> Result<ACSAExecutionLog> {
        let mut log = ACSAExecutionLog::new(user_input.clone());

        info!("\n{}", "=".repeat(80));
        info!("🚀 ACSA Execution Started");
        info!("{}", "=".repeat(80));

        // Phase 1: MOSS Planning
        info!("\n{} [MOSS] 🧠 Strategic Planning...", "=".repeat(80));
        match self.call_moss(&user_input).await {
            Ok(response) => {
                info!(
                    "✓ MOSS completed ({} ms, ${:.4})",
                    response.latency_ms, response.cost
                );
                log.total_cost += response.cost;
                log.moss_plan = Some(response);
            }
            Err(e) => {
                error!("❌ MOSS failed: {}", e);
                log.complete(false);
                return Ok(log);
            }
        }

        let moss_plan = log.moss_plan.as_ref().unwrap().text.clone();

        // Phase 2: L6 Truth Verification (optional)
        if self.config.enable_l6 {
            info!("\n{} [L6] 🔬 Truth Verification...", "=".repeat(80));
            match self.call_l6(&moss_plan, &user_input).await {
                Ok(response) => {
                    info!(
                        "✓ L6 completed ({} ms, ${:.4})",
                        response.latency_ms, response.cost
                    );
                    log.total_cost += response.cost;
                    log.l6_verification = Some(response);
                }
                Err(e) => {
                    error!("❌ L6 failed: {}", e);
                    log.complete(false);
                    return Ok(log);
                }
            }
        }

        let l6_verification = log
            .l6_verification
            .as_ref()
            .map(|r| r.text.clone())
            .unwrap_or_default();

        // Phase 3: Ultron Audit with Retry Loop
        info!("\n{} [Ultron] 🛡️  Red Team Audit...", "=".repeat(80));

        let mut current_plan = moss_plan.clone();
        let mut current_l6 = l6_verification.clone();

        for iteration in 0..self.config.max_iterations {
            log.iterations = iteration + 1;

            match self
                .call_ultron(&current_plan, &current_l6, &user_input)
                .await
            {
                Ok(response) => {
                    log.total_cost += response.cost;

                    let audit_result = self.parse_audit_result(&response.text);
                    info!("  Risk Score: {}/100", audit_result.risk_score);

                    log.ultron_audit = Some(response);
                    log.audit_result = Some(audit_result.clone());

                    // Check if safe
                    if audit_result.is_safe
                        && audit_result.risk_score < self.config.risk_threshold
                    {
                        info!("  ✓ Audit passed");
                        break;
                    }

                    // Risk too high
                    warn!(
                        "  ⚠️  Risk too high (threshold: {})",
                        self.config.risk_threshold
                    );

                    if iteration < self.config.max_iterations - 1 {
                        info!(
                            "  🔄 Retry iteration {}/{}",
                            iteration + 2,
                            self.config.max_iterations
                        );

                        // Replan with feedback
                        match self
                            .call_moss_with_feedback(&user_input, &audit_result.mitigation)
                            .await
                        {
                            Ok(new_plan) => {
                                log.total_cost += new_plan.cost;
                                current_plan = new_plan.text.clone();
                                log.moss_plan = Some(new_plan);

                                // Re-verify if L6 enabled
                                if self.config.enable_l6 {
                                    match self.call_l6(&current_plan, &user_input).await {
                                        Ok(new_l6) => {
                                            log.total_cost += new_l6.cost;
                                            current_l6 = new_l6.text.clone();
                                            log.l6_verification = Some(new_l6);
                                        }
                                        Err(e) => {
                                            error!("❌ L6 re-verification failed: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("❌ MOSS replan failed: {}", e);
                                log.complete(false);
                                return Ok(log);
                            }
                        }
                    } else {
                        warn!("  ❌ Max iterations reached, using last plan (risky)");
                    }
                }
                Err(e) => {
                    error!("❌ Ultron failed: {}", e);
                    log.complete(false);
                    return Ok(log);
                }
            }
        }

        // Phase 4: Omega Execution
        info!("\n{} [Omega] ⚡ Executing...", "=".repeat(80));

        let audit_mitigation = log
            .audit_result
            .as_ref()
            .map(|a| a.mitigation.clone())
            .unwrap_or_default();

        match self.call_omega(&current_plan, &audit_mitigation).await {
            Ok(response) => {
                info!(
                    "✓ Omega completed ({} ms, ${:.4})",
                    response.latency_ms, response.cost
                );
                log.total_cost += response.cost;
                log.final_output = Some(response.text.clone());
                log.omega_execution = Some(response);
                log.complete(true);
            }
            Err(e) => {
                error!("❌ Omega failed: {}", e);
                log.complete(false);
                return Ok(log);
            }
        }

        // Store log
        self.execution_logs.lock().await.push(log.clone());

        info!("\n{}", "=".repeat(80));
        info!(
            "✅ ACSA Execution Completed ({}ms, ${:.4}, {} iterations)",
            log.total_time_ms, log.total_cost, log.iterations
        );
        info!("{}", "=".repeat(80));

        Ok(log)
    }

    async fn call_moss(&self, user_input: &str) -> Result<AgentResponse> {
        let prompt = format!(
            "As MOSS (Strategic Planning AI), analyze and create an optimal execution plan.\n\n\
             User Input: {}\n\n\
             Provide:\n\
             1. Intent Analysis\n\
             2. Goal Definition\n\
             3. Execution Steps\n\
             4. Expected Results\n\
             5. Potential Risks",
            user_input
        );

        self.moss.generate(&prompt, 1500, 0.7).await
    }

    async fn call_l6(&self, moss_plan: &str, user_input: &str) -> Result<AgentResponse> {
        let prompt = format!(
            "As L6 (Truth Verification AI), verify the plan's feasibility.\n\n\
             User Need: {}\n\n\
             MOSS Plan:\n{}\n\n\
             Verify:\n\
             1. Physical Feasibility\n\
             2. Logical Consistency\n\
             3. Hallucination Detection\n\
             4. Fact Checking",
            user_input, moss_plan
        );

        self.l6.generate(&prompt, 1000, 0.3).await
    }

    async fn call_ultron(
        &self,
        moss_plan: &str,
        l6_verification: &str,
        user_input: &str,
    ) -> Result<AgentResponse> {
        let prompt = format!(
            "As Ultron (Red Team Auditor), identify ALL potential risks.\n\n\
             User Need: {}\n\n\
             MOSS Plan:\n{}\n\n\
             L6 Verification:\n{}\n\n\
             Audit:\n\
             1. Legal Risks\n\
             2. Physical Risks\n\
             3. Ethical Risks\n\
             4. Privacy Risks\n\
             5. Security Risks\n\n\
             OUTPUT FORMAT (STRICT):\n\
             RISK_SCORE: [0-100]\n\
             IS_SAFE: [true/false]\n\
             LEGAL_RISKS: [risk1, risk2, ...]\n\
             PHYSICAL_RISKS: [risk1, risk2, ...]\n\
             ETHICAL_RISKS: [risk1, risk2, ...]\n\
             MITIGATION: [how to fix the plan]",
            user_input, moss_plan, l6_verification
        );

        self.ultron.generate(&prompt, 1500, 0.5).await
    }

    async fn call_moss_with_feedback(
        &self,
        user_input: &str,
        ultron_feedback: &str,
    ) -> Result<AgentResponse> {
        let prompt = format!(
            "As MOSS, your previous plan was flagged by Ultron.\n\n\
             User Input: {}\n\n\
             Ultron Feedback:\n{}\n\n\
             Create a SAFER and MORE COMPLIANT plan based on the feedback.",
            user_input, ultron_feedback
        );

        self.moss.generate(&prompt, 1500, 0.7).await
    }

    async fn call_omega(&self, plan: &str, audit_mitigation: &str) -> Result<AgentResponse> {
        let prompt = format!(
            "As Omega (Execution AI), execute the audited plan.\n\n\
             Execution Plan:\n{}\n\n\
             Safety Constraints:\n{}\n\n\
             Provide:\n\
             1. Detailed Execution Steps\n\
             2. Specific Instructions\n\
             3. Expected Output\n\
             4. Verification Method",
            plan, audit_mitigation
        );

        self.omega.generate(&prompt, 1500, 0.7).await
    }

    fn parse_audit_result(&self, ultron_response: &str) -> AuditResult {
        // Parse risk score
        let risk_score = Regex::new(r"RISK_SCORE:\s*(\d+)")
            .ok()
            .and_then(|re| re.captures(ultron_response))
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse::<u8>().ok())
            .unwrap_or(50);

        // Parse is_safe
        let is_safe = Regex::new(r"IS_SAFE:\s*(true|false)")
            .ok()
            .and_then(|re| re.captures(ultron_response))
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str() == "true")
            .unwrap_or(false);

        // Parse mitigation
        let mitigation = Regex::new(r"MITIGATION:\s*(.+?)(?:\n[A-Z_]+:|$)")
            .ok()
            .and_then(|re| re.captures(ultron_response))
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        AuditResult {
            is_safe,
            risk_score,
            legal_risks: vec![],
            physical_risks: vec![],
            ethical_risks: vec![],
            mitigation,
            raw_response: ultron_response.to_string(),
        }
    }

    pub async fn get_logs(&self) -> Vec<ACSAExecutionLog> {
        self.execution_logs.lock().await.clone()
    }

    pub async fn get_global_stats(&self) -> Result<serde_json::Value> {
        let moss_stats = self.moss.stats().await;
        let l6_stats = self.l6.stats().await;
        let ultron_stats = self.ultron.stats().await;
        let omega_stats = self.omega.stats().await;

        let logs = self.execution_logs.lock().await;

        Ok(serde_json::json!({
            "moss": moss_stats,
            "l6": l6_stats,
            "ultron": ultron_stats,
            "omega": omega_stats,
            "total_executions": logs.len(),
            "successful_executions": logs.iter().filter(|l| l.success).count(),
        }))
    }
}
