use anyhow::Result;
use tracing::trace;

use crate::{
    cascades::{
        optimizer::{CascadesOptimizer, ExprId},
        tasks::{ApplyRuleTask, ExploreGroupTask},
        Memo,
    },
    nodes::NodeType,
    rules::RuleMatcher,
};

use super::Task;

pub struct OptimizeExpressionTask {
    expr_id: ExprId,
    exploring: bool,
}

impl OptimizeExpressionTask {
    pub fn new(expr_id: ExprId, exploring: bool) -> Self {
        Self { expr_id, exploring }
    }
}

fn top_matches<T: NodeType>(matcher: &RuleMatcher<T>, match_typ: T) -> bool {
    match matcher {
        RuleMatcher::MatchNode { typ, .. } => typ == &match_typ,
        RuleMatcher::MatchDiscriminant {
            typ_discriminant, ..
        } => std::mem::discriminant(&match_typ) == *typ_discriminant,
        _ => panic!("IR should have root node of match"),
    }
}

impl<T: NodeType, M: Memo<T>> Task<T, M> for OptimizeExpressionTask {
    fn execute(&self, optimizer: &mut CascadesOptimizer<T, M>) -> Result<Vec<Box<dyn Task<T, M>>>> {
        let expr = optimizer.get_expr_memoed(self.expr_id);
        trace!(event = "task_begin", task = "optimize_expr", expr_id = %self.expr_id, expr = %expr);
        let mut tasks = vec![];
        for (rule_id, rule_wrapper) in optimizer.rules().iter().enumerate() {
            let rule = rule_wrapper.rule();
            if optimizer.is_rule_fired(self.expr_id, rule_id) {
                continue;
            }
            // Skip impl rules when exploring
            if self.exploring && rule.is_impl_rule() {
                continue;
            }
            // Skip transformation rules when budget is used
            if optimizer.ctx.budget_used && !rule.is_impl_rule() {
                continue;
            }
            if top_matches(rule.matcher(), expr.typ.clone()) {
                tasks.push(
                    Box::new(ApplyRuleTask::new(rule_id, self.expr_id, self.exploring))
                        as Box<dyn Task<T, M>>,
                );
                for &input_group_id in &expr.children {
                    tasks.push(
                        Box::new(ExploreGroupTask::new(input_group_id)) as Box<dyn Task<T, M>>
                    );
                }
            }
        }
        trace!(event = "task_end", task = "optimize_expr", expr_id = %self.expr_id);
        Ok(tasks)
    }

    fn describe(&self) -> String {
        format!("optimize_expr {}", self.expr_id)
    }
}
