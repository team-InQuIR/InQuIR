from qiskit import QuantumRegister, ClassicalRegister, QuantumCircuit
from qiskit.transpiler import PassManager
from qiskit.transpiler.passes import Unroller

def adder(num_q, decomposition = True):
    # returns qasm of quantum circuit with num_q * 3 qubits
    A = QuantumRegister(num_q, 'inputA')
    B = QuantumRegister(num_q, 'inputB')
    carry = QuantumRegister(num_q, 'carry')
    sum_a_b = ClassicalRegister(num_q, 'sumab')
    qc = QuantumCircuit(A,B,carry, sum_a_b)

    # calc 2^0 place
    qc.ccx(A[-1],B[-1],carry[-1])
    qc.cx(A[-1],B[-1])
    qc.cx(carry[-1],B[-1])
    qc.barrier()
    
    # calc 2^(i-1) place
    for i in range(2, num_q+1):
        qc.ccx(A[-i], carry[-i+1], carry[-i])
        qc.ccx(B[-i], carry[-i+1], carry[-i])
        qc.ccx(A[-i], B[-i], carry[-i])
        qc.cx(carry[-i+1], B[-i])
        qc.cx(A[-i],B[-i])
        qc.barrier()
    qc.measure(B, sum_a_b)
    
    # decompose gates into <cx,u>
    if decomposition == True:
        pass_ = Unroller(["cx", "u"])
        pm = PassManager(pass_)
        qc = pm.run(qc)
    return qc