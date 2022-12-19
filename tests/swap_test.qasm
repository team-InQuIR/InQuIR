OPENQASM 2.0;
qreg q[3];
creg q[3];
cx q[0],q[2];
cx q[2],q[0];
measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];
