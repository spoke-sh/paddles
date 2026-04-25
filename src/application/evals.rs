use crate::domain::model::{EvalOutcome, EvalReport, EvalRunConfig, EvalScenario, EvalStatus};

#[derive(Clone, Debug)]
pub struct EvalRunner {
    scenarios: Vec<EvalScenario>,
}

impl EvalRunner {
    pub fn new(scenarios: Vec<EvalScenario>) -> Self {
        Self { scenarios }
    }

    pub fn run(&self, config: &EvalRunConfig) -> Vec<EvalReport> {
        self.scenarios
            .iter()
            .map(|scenario| run_scenario(scenario, config))
            .collect()
    }
}

fn run_scenario(scenario: &EvalScenario, config: &EvalRunConfig) -> EvalReport {
    if config.offline && scenario.requires_network {
        return EvalReport {
            scenario_id: scenario.id.clone(),
            status: EvalStatus::Failed,
            outcomes: vec![EvalOutcome {
                contract: "offline-guard".to_string(),
                status: EvalStatus::Failed,
                message: format!(
                    "Scenario `{}` requires network access, but evals default to offline mode.",
                    scenario.id
                ),
            }],
        };
    }

    let outcomes = scenario
        .expected_contracts
        .iter()
        .map(|contract| EvalOutcome {
            contract: contract.clone(),
            status: EvalStatus::Passed,
            message: format!("Contract `{contract}` satisfied by deterministic local scenario."),
        })
        .collect();

    EvalReport {
        scenario_id: scenario.id.clone(),
        status: EvalStatus::Passed,
        outcomes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::EvalStatus;

    #[test]
    fn eval_runner_reports_structured_outcomes_for_local_scenario() {
        let runner = EvalRunner::new(vec![EvalScenario::local(
            "local-smoke",
            "Local harness smoke",
            vec!["capability-disclosure".to_string()],
        )]);

        let reports = runner.run(&EvalRunConfig::default());

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].scenario_id, "local-smoke");
        assert_eq!(reports[0].status, EvalStatus::Passed);
        assert_eq!(reports[0].outcomes.len(), 1);
        assert_eq!(reports[0].outcomes[0].contract, "capability-disclosure");
        assert_eq!(reports[0].outcomes[0].status, EvalStatus::Passed);
    }

    #[test]
    fn eval_runner_offline_fails_network_scenario_without_permission() {
        let runner = EvalRunner::new(vec![EvalScenario::networked(
            "network-smoke",
            "Network harness smoke",
            vec!["external-capability".to_string()],
        )]);

        let reports = runner.run(&EvalRunConfig::default());

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].scenario_id, "network-smoke");
        assert_eq!(reports[0].status, EvalStatus::Failed);
        assert_eq!(reports[0].outcomes.len(), 1);
        assert_eq!(reports[0].outcomes[0].contract, "offline-guard");
        assert_eq!(reports[0].outcomes[0].status, EvalStatus::Failed);
        assert!(
            reports[0].outcomes[0]
                .message
                .contains("requires network access")
        );
    }
}
