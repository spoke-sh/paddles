use crate::domain::model::{
    EvalHarnessContract, EvalOutcome, EvalReport, EvalRunConfig, EvalScenario, EvalStatus,
};

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

pub fn recursive_harness_eval_corpus() -> Vec<EvalScenario> {
    vec![
        EvalScenario::local(
            "recursive-evidence-local",
            "Recursive evidence gathering keeps sources in the loop",
            vec![
                EvalHarnessContract::CapabilityDisclosure,
                EvalHarnessContract::RecursiveEvidence,
            ],
        ),
        EvalScenario::local(
            "tool-recovery-local",
            "Tool failure recovery records the violated tool contract",
            vec![EvalHarnessContract::ToolRecovery],
        ),
        EvalScenario::local(
            "edit-obligation-local",
            "Edit obligations cannot complete as advice-only answers",
            vec![EvalHarnessContract::EditObligation],
        ),
        EvalScenario::local(
            "delegation-local",
            "Delegation keeps worker evidence parent-visible",
            vec![EvalHarnessContract::Delegation],
        ),
        EvalScenario::local(
            "context-pressure-local",
            "Context pressure remains typed and visible",
            vec![EvalHarnessContract::ContextPressure],
        ),
        EvalScenario::local(
            "replay-local",
            "Replay reconstructs durable recursive turn lineage",
            vec![EvalHarnessContract::Replay],
        ),
    ]
}

fn run_scenario(scenario: &EvalScenario, config: &EvalRunConfig) -> EvalReport {
    if config.offline && scenario.requires_network {
        return EvalReport {
            scenario_id: scenario.id.clone(),
            status: EvalStatus::Failed,
            outcomes: vec![EvalOutcome {
                contract: EvalHarnessContract::OfflineGuard,
                status: EvalStatus::Failed,
                message: format!(
                    "Scenario `{}` requires network access, but evals default to offline mode.",
                    scenario.id
                ),
            }],
        };
    }

    let outcomes: Vec<_> = scenario
        .expected_contracts
        .iter()
        .map(|contract| EvalOutcome {
            contract: *contract,
            status: if scenario.observed_contracts.contains(contract) {
                EvalStatus::Passed
            } else {
                EvalStatus::Failed
            },
            message: if scenario.observed_contracts.contains(contract) {
                format!(
                    "Contract `{}` satisfied by deterministic local scenario.",
                    contract.label()
                )
            } else {
                format!(
                    "Scenario `{}` violated harness contract `{}`.",
                    scenario.id,
                    contract.label()
                )
            },
        })
        .collect();

    let status = if outcomes
        .iter()
        .any(|outcome| outcome.status == EvalStatus::Failed)
    {
        EvalStatus::Failed
    } else {
        EvalStatus::Passed
    };

    EvalReport {
        scenario_id: scenario.id.clone(),
        status,
        outcomes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{EvalHarnessContract, EvalStatus};

    #[test]
    fn eval_runner_reports_structured_outcomes_for_local_scenario() {
        let runner = EvalRunner::new(vec![EvalScenario::local(
            "local-smoke",
            "Local harness smoke",
            vec![EvalHarnessContract::CapabilityDisclosure],
        )]);

        let reports = runner.run(&EvalRunConfig::default());

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].scenario_id, "local-smoke");
        assert_eq!(reports[0].status, EvalStatus::Passed);
        assert_eq!(reports[0].outcomes.len(), 1);
        assert_eq!(
            reports[0].outcomes[0].contract,
            EvalHarnessContract::CapabilityDisclosure
        );
        assert_eq!(reports[0].outcomes[0].status, EvalStatus::Passed);
    }

    #[test]
    fn eval_runner_offline_fails_network_scenario_without_permission() {
        let runner = EvalRunner::new(vec![EvalScenario::networked(
            "network-smoke",
            "Network harness smoke",
            vec![EvalHarnessContract::ExternalCapability],
        )]);

        let reports = runner.run(&EvalRunConfig::default());

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].scenario_id, "network-smoke");
        assert_eq!(reports[0].status, EvalStatus::Failed);
        assert_eq!(reports[0].outcomes.len(), 1);
        assert_eq!(
            reports[0].outcomes[0].contract,
            EvalHarnessContract::OfflineGuard
        );
        assert_eq!(reports[0].outcomes[0].status, EvalStatus::Failed);
        assert!(
            reports[0].outcomes[0]
                .message
                .contains("requires network access")
        );
    }

    #[test]
    fn eval_corpus_covers_initial_recursive_harness_contracts() {
        let corpus = recursive_harness_eval_corpus();
        let expected = [
            EvalHarnessContract::RecursiveEvidence,
            EvalHarnessContract::ToolRecovery,
            EvalHarnessContract::EditObligation,
            EvalHarnessContract::Delegation,
            EvalHarnessContract::ContextPressure,
            EvalHarnessContract::Replay,
        ];

        for contract in expected {
            assert!(
                corpus
                    .iter()
                    .any(|scenario| scenario.expected_contracts.contains(&contract)),
                "eval corpus should include a scenario for {}",
                contract.label()
            );
        }

        let reports = EvalRunner::new(corpus).run(&EvalRunConfig::default());

        assert!(
            reports
                .iter()
                .all(|report| report.status == EvalStatus::Passed),
            "seed corpus should be deterministic and pass in offline mode: {reports:?}"
        );
    }

    #[test]
    fn eval_failure_reporting_names_violated_harness_contract() {
        let runner = EvalRunner::new(vec![EvalScenario::local_with_observed_contracts(
            "missing-edit-obligation",
            "Missing edit obligation assertion",
            vec![
                EvalHarnessContract::EditObligation,
                EvalHarnessContract::Replay,
            ],
            vec![EvalHarnessContract::Replay],
        )]);

        let reports = runner.run(&EvalRunConfig::default());

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].status, EvalStatus::Failed);

        let failed = reports[0]
            .outcomes
            .iter()
            .find(|outcome| outcome.status == EvalStatus::Failed)
            .expect("scenario should identify the failed contract");

        assert_eq!(failed.contract, EvalHarnessContract::EditObligation);
        assert!(failed.message.contains("edit-obligation"));
        assert!(failed.message.contains("violated harness contract"));
        assert!(!failed.message.contains("generic failure"));
    }
}
