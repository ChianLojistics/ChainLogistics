use crate::error::AppError;
use crate::rules::dsl::{Rule, RuleSet};
use crate::rules::actions::{ActionExecutor, ActionResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RuleEngine {
    rule_sets: Arc<RwLock<Vec<RuleSet>>>,
    executor: Arc<ActionExecutor>,
}

impl RuleEngine {
    pub fn new(executor: ActionExecutor) -> Self {
        Self {
            rule_sets: Arc::new(RwLock::new(Vec::new())),
            executor: Arc::new(executor),
        }
    }

    pub async fn add_rule_set(&self, rule_set: RuleSet) -> Result<(), AppError> {
        let mut rule_sets = self.rule_sets.write().await;
        rule_sets.push(rule_set);
        Ok(())
    }

    pub async fn remove_rule_set(&self, id: &str) -> Result<(), AppError> {
        let mut rule_sets = self.rule_sets.write().await;
        rule_sets.retain(|rs| rs.id != id);
        Ok(())
    }

    pub async fn evaluate(&self, context: &HashMap<String, serde_json::Value>) -> Result<Vec<MatchedRule>, AppError> {
        let rule_sets = self.rule_sets.read().await;
        let mut matched_rules = Vec::new();

        for rule_set in rule_sets.iter() {
            let rules = rule_set.evaluate(context)?;
            for rule in rules {
                matched_rules.push(MatchedRule {
                    rule_set_id: rule_set.id.clone(),
                    rule: rule.clone(),
                });
            }
        }

        // Sort by priority
        matched_rules.sort_by(|a, b| b.rule.priority.cmp(&a.rule.priority));

        Ok(matched_rules)
    }

    pub async fn execute_matched_rules(&self, context: &HashMap<String, serde_json::Value>) -> Result<Vec<ActionResult>, AppError> {
        let matched_rules = self.evaluate(context).await?;
        let mut results = Vec::new();

        for matched_rule in matched_rules {
            for action in &matched_rule.rule.actions {
                let result = self.executor.execute(action, context).await?;
                results.push(result);
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct MatchedRule {
    pub rule_set_id: String,
    pub rule: Rule,
}

impl RuleEngine {
    pub async fn get_rule_sets(&self) -> Vec<RuleSet> {
        self.rule_sets.read().await.clone()
    }

    pub async fn get_rule_set(&self, id: &str) -> Option<RuleSet> {
        let rule_sets = self.rule_sets.read().await;
        rule_sets.iter().find(|rs| rs.id == id).cloned()
    }
}
