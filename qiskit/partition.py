import numpy as np
import random
import matplotlib.pyplot as plt
from math import pi
import networkx as nx
from networkx.algorithms.community import kernighan_lin_bisection

from qiskit import qasm
from qiskit import QuantumRegister, ClassicalRegister, QuantumCircuit
from qiskit.transpiler import PassManager
from qiskit.transpiler.passes import Unroller

def qasm_to_uwg(qasm_filename):
    print('converting qasm to uwg')
    f = open(qasm_filename, "r")
    qasm = f.readlines()
    # Connectivity of qc
    G = nx.Graph()
    # quantum registers
    qreg = []
    # size of quantum register(s)
    qreg_size = []

    for instruction in qasm:
        instruction = instruction.split(';')[0]
        if instruction.startswith('OPENQASM'):
            pass
        elif instruction.startswith('include'):
            pass
        elif instruction.startswith('qreg'):
            # list of quantum register(s)
            instruction = instruction.strip("qreg")
            # print(str(instruction))
            splits = instruction.split('[')
            regname = splits[0]
            regname = regname.strip()
            qreg.append(regname)
            size = splits[1].strip(']')
            qreg_size.append(size)
            for i in range(int(size)):
                G.add_node(regname+'['+str(i)+']')
        elif instruction.startswith('cx'):
            instruction = instruction.strip("cx")
            splits = instruction.split(',')
            control_qubit = splits[0].strip()
            target_qqubit = splits[1].strip()
            # print(control_qubit, target_qubit)
            if G.has_edge(control_qubit, target_qqubit):
                G[control_qubit][target_qqubit]['weight'] += 1
            else:
                G.add_edge(control_qubit, target_qqubit, weight=1)
        elif instruction.startswith('creg') or instruction.startswith('u') or instruction.startswith('measure'):
            # It doesn't affect to connectivity graph
            pass
        else:
            print("Incompatible instruction!: " + str(instruction))
    return [G,qreg,qreg_size]

def qasm_to_inq(qasm_filename):
    # partition given by the form like [[q0,q1],[q2,q3],[q4,q5]]
    uwgs = qasm_to_uwg(qasm_filename)
    print('converting qasm to inq')
    partition = kernighan_lin_bisection(uwgs[0], partition=None, max_iter=10, weight='weight', seed=None)
    partition = [list(partition[0]),list(partition[1])]

    inq = {}
    # processor num
    p_n = 0
    for i in partition:
        nodeinit = []
        nodeinit.append("node" + str(p_n) + "{")
        for j in i:
            nodeinit.append( str(j) + " = init();")
        inq[p_n] = nodeinit
        p_n += 1

    f = open(qasm_filename, "r")
    qasm = f.readlines()

    # RCX num
    rcx_num = 0

    # declaration for classical reg
    decls = []

    for instruction in qasm:
        instruction = instruction.split(';')[0]
        if instruction.startswith('OPENQASM'):
            pass
        elif instruction.startswith('qreg') or instruction.startswith('include'):
            pass
        elif instruction.startswith('creg'):
            instruction = instruction.strip("creg")
            instruction = instruction.strip("]")
            splits = instruction.split('[')
            cr = splits[0]
            cr_num_bits = splits[1]
            decls.append('CR' + str(cr) + ',' + str(cr_num_bits) + ';')
        elif instruction.startswith('cx'):
            instruction = instruction.strip("cx")
            splits = instruction.split(',')
            control_qubit = splits[0].strip()
            target_qubit = splits[1].strip()
            # c_index = 0
            # t_index = 0
            for processor in partition:
                if control_qubit in processor:
                    c_index = partition.index(processor)
                else:
                    for pr_node in partition:
                        if control_qubit in pr_node:
                            c_index = partition.index(pr_node)
                        else:
                            pass
                if target_qubit in processor:
                    t_index =partition.index(processor)
                else:
                    for pr_node in partition:
                        if target_qubit in pr_node:
                            t_index = partition.index(pr_node)
                        else:
                            pass
                if c_index == t_index:
                    # Internal CX
                    # CX ["qa", "qb"];
                    inq[c_index].append("cx " + str(control_qubit) +", "+ str(target_qubit) +  ";")
                else:
                    # RCX
                    inq[c_index].append("rcxc " + str(control_qubit) + ", " + str(t_index) + ", rcxc[" + str(rcx_num) + "];")
                    inq[t_index].append("rcxt " + str(target_qubit) + ", " + str(c_index) + ", rcxt[" + str(rcx_num) + "];")
                    rcx_num += 1
        elif instruction.startswith('u'):
            # u(theta,phi,lambda) qreg[qubit];
            instruction = instruction.strip("u(")
            splits = instruction.split()
            qubit = splits[1]
            params = splits[0].strip(")")
            p_index = 0
            for p in partition:
                if qubit in p:
                    inq[p_index].append("u(" + str(params) + ") " + str(qubit) + ";")
                else:
                    p_index += 1
        elif instruction.startswith('measure'):
            # measure qreg[qubit] -> qreg[qubit]
            instruction = instruction.strip("measure ")
            splits = instruction.split(' -> ')
            measured_qubit = splits[0].replace(' ', '')
            measured_result = splits[1].replace(' ', '')
            # print('measured_qubit: ' + str(splits[0]))
            # print('measured_result: ' + str(splits[1]))
            for processor in partition:
                if measured_qubit in processor:
                    q_index = partition.index(processor)
            # inq[q_index].append("measure " + str(measured_qubit) +" -> "+ str(measured_result) +  ";")
            inq[q_index].append(str(measured_result) + " = measure(" + str(measured_qubit)+  ");")
        else:
            print("Incompatible instruction!: " + str(instruction))


        if qasm_filename.endswith('.qasm'):
            qasm_filename = qasm_filename[:-5]
        elif qasm_filename.endswith('.inq'):
            qasm_filename = qasm_filename[:-4]

        path_w = qasm_filename + '.inq'
        with open(path_w, mode='w') as f:
            lines = 0
            # declaration for classical registers
            # for decl in decls:
            #         f.write(str(decl)+ '\n')
            #         lines += 1
            for node in inq.keys():
                for inst in inq[node]:
                    f.write(str(inst) + '\n')
                    lines += 1
                f.write('}\n')
    return qasm_filename + ".inq generated. " + str(lines - 1) + " lines."
