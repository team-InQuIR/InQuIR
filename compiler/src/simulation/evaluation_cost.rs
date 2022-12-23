#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EvaluationCost {
    time: u32,
    e_depth: u32,
    c_depth: u32,
}
impl EvaluationCost {
    pub fn new(time: u32, e_depth: u32, c_depth: u32) -> Self {
        Self {
            time,
            e_depth,
            c_depth,
        }
    }

    pub fn time(&self) -> u32 {
        self.time
    }

    pub fn e_depth(&self) -> u32 {
        self.e_depth
    }

    pub fn c_depth(&self) -> u32 {
        self.c_depth
    }

    pub fn add_time(&mut self, v: u32) {
        self.time += v;
    }

    pub fn add_e_depth(&mut self, v: u32) {
        self.e_depth += v;
    }

    pub fn add_c_depth(&mut self, v: u32) {
        self.c_depth += v;
    }
}

pub fn collect_cost(costs: Vec<EvaluationCost>) -> EvaluationCost {
    let mut time = 0;
    let mut e_depth = 0;
    let mut c_depth = 0;
    costs.into_iter().for_each(|cost| {
        time = u32::max(time, cost.time());
        e_depth = u32::max(e_depth, cost.e_depth());
        c_depth = u32::max(c_depth, cost.c_depth());
    });
    EvaluationCost::new(time, e_depth, c_depth)
}
