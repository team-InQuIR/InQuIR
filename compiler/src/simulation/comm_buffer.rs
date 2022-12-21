use inquir::{
    Label,
    Value,
};
use std::collections::{VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendData {
    time: u32,
    label: Label,
    value: Value,
}

impl SendData {
    pub fn new(time: u32, label: Label, value: Value) -> Self {
        Self {
            time,
            label,
            value
        }
    }

    pub fn time(&self) -> u32 {
        self.time
    }

    pub fn label(&self) -> &Label {
        &self.label
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

    pub fn pop(&mut self, l: Label) -> Option<(u32, Value)> {
        let idx = self.que.iter().position(|data| *data.label() == l);
        idx.and_then(|idx| self.que.remove(idx)).map(|data| (data.time(), data.value().clone()))
    }
}
