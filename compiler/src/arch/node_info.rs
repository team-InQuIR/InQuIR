use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct NodeInfo {
    /// The number of data qubits.
    data_qubits: u32,
    /// The number of communication qubits.
    comm_qubits: u32,
    #[serde(default = "default_single_gate_cost")]
    single_gate_cost: u64,
    #[serde(default = "default_local_cx_cost")]
    local_cx_cost: u64,
    #[serde(default = "default_gen_ent_cost")]
    gen_ent_cost: u64,
    #[serde(default = "default_measure_cost")]
    measure_cost: u64,
    #[serde(default = "default_classical_comm_cost")]
    classical_comm_cost: u64,
}

impl NodeInfo {
    pub fn num_of_data_qubits(&self) -> u32 {
        self.data_qubits
    }

    pub fn num_of_comm_qubits(&self) -> u32 {
        self.comm_qubits
    }

    pub fn single_gate_cost(&self) -> u64 {
        self.single_gate_cost
    }

    pub fn local_cx_cost(&self) -> u64 {
        self.local_cx_cost
    }

    pub fn gen_ent_cost(&self) -> u64 {
        self.gen_ent_cost
    }

    pub fn measure_cost(&self) -> u64 {
        self.measure_cost
    }

    pub fn classical_comm_cost(&self) -> u64 {
        self.classical_comm_cost
    }
}

/*
Sung, Y. et al. Realization of High-Fidelity CZ and $ZZ$-Free iSWAP Gates with a Tunable Coupler. Phys. Rev. X 11, 021058 (2021).
 */
fn default_single_gate_cost() -> u64 {
    30 // 30 [ns]
}

/*
Wei, K. X. et al. Hamiltonian Engineering with Multicolor Drives for Fast Entangling Gates and Quantum Crosstalk Cancellation. Phys. Rev. Lett. 129, 060501 (2022).
 */
fn default_local_cx_cost() -> u64 {
    60 // 60 [ns]
}

/*
Ang, J. et al. Architectures for Multinode Superconducting Quantum Computers. Preprint at https://doi.org/10.48550/arXiv.2212.06167 (2022).
 */
fn default_gen_ent_cost() -> u64 {
    1000 // 1 [micro sec]
}

/*
Sunada, Y. et al. Fast Readout and Reset of a Superconducting Qubit Coupled to a Resonator with an Intrinsic Purcell Filter. Phys. Rev. Appl. 17, 044016 (2022).
 */
fn default_measure_cost() -> u64 {
    240 // 240 [ns]
}

fn default_classical_comm_cost() -> u64 {
    30
}

