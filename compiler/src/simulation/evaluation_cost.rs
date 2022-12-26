#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct EvaluationCost {
    total_time: u64,
    gen_ent_time: u64,
    e_depth: u64,
    c_depth: u64,
}
impl EvaluationCost {
    pub fn new(total_time: u64, gen_ent_time: u64, e_depth: u64, c_depth: u64) -> Self {
        Self {
            total_time,
            gen_ent_time,
            e_depth,
            c_depth,
        }
    }

    pub fn total_time(&self) -> u64 {
        self.total_time
    }

    pub fn gen_ent_time(&self) -> u64 {
        self.gen_ent_time
    }

    pub fn e_depth(&self) -> u64 {
        self.e_depth
    }

    pub fn c_depth(&self) -> u64 {
        self.c_depth
    }

    pub fn add_gen_ent_time(&mut self, v: u64) {
        self.total_time += v;
        self.gen_ent_time += v;

    }

    pub fn add_total_time(&mut self, v: u64) {
        self.total_time += v;
    }

    pub fn add_e_depth(&mut self, v: u64) {
        self.e_depth += v;
    }

    pub fn add_c_depth(&mut self, v: u64) {
        self.c_depth += v;
    }
}

pub fn collect_cost(costs: Vec<EvaluationCost>) -> EvaluationCost {
    let mut total_time = 0;
    let mut gen_ent_time = 0;
    let mut e_depth = 0;
    let mut c_depth = 0;
    costs.into_iter().for_each(|cost| {
        total_time = u64::max(total_time, cost.total_time());
        gen_ent_time = u64::max(total_time, cost.gen_ent_time);
        e_depth = u64::max(e_depth, cost.e_depth());
        c_depth = u64::max(c_depth, cost.c_depth());
    });
    EvaluationCost::new(total_time, gen_ent_time, e_depth, c_depth)
}
