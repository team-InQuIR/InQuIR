use inquir::{
    Label,
    Value,
};
use std::collections::{VecDeque};
use crate::simulation::evaluation_cost::EvaluationCost;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendData {
    label: Label,
    cost: EvaluationCost,
    value: Value,
}

impl SendData {
    pub fn new(label: Label, cost: EvaluationCost, value: Value) -> Self {
        Self {
            label,
            cost,
            value
        }
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn cost(&self) -> EvaluationCost {
        self.cost.clone()
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct CommBuffer {
    que: VecDeque<SendData>,
}

impl CommBuffer {
    pub fn new() -> Self {
        Self {
            que: VecDeque::new(),
        }
    }

    pub fn push(&mut self, data: SendData) {
        self.que.push_back(data);
    }

    pub fn pop(&mut self, l: Label) -> Option<SendData> {
        let idx = self.que.iter().position(|data| *data.label() == l);
        idx.and_then(|idx| self.que.remove(idx))
    }
}
