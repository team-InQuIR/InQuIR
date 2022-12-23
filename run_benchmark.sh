#!/bin/bash

outdir=results

run_qasm() {
    local target=$1
    local config=$2
    local config_name=`basename ${config%.json}`
    local base=`basename ${target%.qasm}`
    #local strategies=("teledata-only" "telegate-only")
    local strategies=("telegate-only")
    for ((j=0; j<${#strategies[@]}; j++))
    do
        local st=${strategies[j]}
        local out="${base}_${st}_${config_name}.inq"
        local met="${base}_${st}_${config_name}.json"
        echo "Compiling ${target} for ${out}"
        ./target/release/inqcc ${target} -o ${outdir}/${out} --config ${config} --strategy ${st} --metrics ${outdir}/${met}
    done
}

targets=(
    'benchmark/quantum_compiler_optim/examples/4gt12-v1_89.qasm'
    'benchmark/quantum_compiler_optim/examples/4gt12-v1_89.qasm'
    'benchmark/quantum_compiler_optim/examples/4gt12-v1_89.qasm'
    'benchmark/quantum_compiler_optim/examples/4gt12-v1_89.qasm'
    'benchmark/quantum_compiler_optim/examples/9symml_195.qasm'
    'benchmark/quantum_compiler_optim/examples/9symml_195.qasm'
    'benchmark/quantum_compiler_optim/examples/9symml_195.qasm'
    'benchmark/quantum_compiler_optim/examples/9symml_195.qasm'
    'benchmark/quantum_compiler_optim/examples/ising_model_16.qasm'
    'benchmark/quantum_compiler_optim/examples/ising_model_16.qasm'
    'benchmark/quantum_compiler_optim/examples/ising_model_16.qasm'
    'benchmark/quantum_compiler_optim/examples/ising_model_16.qasm'
    'benchmark/quantum_compiler_optim/examples/life_238.qasm'
    'benchmark/quantum_compiler_optim/examples/life_238.qasm'
    'benchmark/quantum_compiler_optim/examples/life_238.qasm'
    'benchmark/quantum_compiler_optim/examples/life_238.qasm'
    'benchmark/quantum_compiler_optim/examples/root_255.qasm'
    'benchmark/quantum_compiler_optim/examples/root_255.qasm'
    'benchmark/quantum_compiler_optim/examples/root_255.qasm'
    'benchmark/quantum_compiler_optim/examples/root_255.qasm'
    'benchmark/quantum_compiler_optim/examples/rd53_138.qasm'
    'benchmark/quantum_compiler_optim/examples/rd53_138.qasm'
    'benchmark/quantum_compiler_optim/examples/rd53_138.qasm'
    'benchmark/quantum_compiler_optim/examples/rd53_138.qasm'
    'benchmark/quantum_compiler_optim/examples/sqn_258.qasm'
    'benchmark/quantum_compiler_optim/examples/sqn_258.qasm'
    'benchmark/quantum_compiler_optim/examples/sqn_258.qasm'
    'benchmark/quantum_compiler_optim/examples/sqn_258.qasm'
    'benchmark/QASMBench/large/qft_n20/qft_n20.qasm'
    'benchmark/QASMBench/large/qft_n20/qft_n20.qasm')
configs=(
    'config/4x4-linear.json'
    'config/4x4-2-linear.json'
    'config/4x4-3-linear.json'
    'config/4x4-4-linear.json'
    'config/4x4-linear.json'
    'config/4x4-2-linear.json'
    'config/4x4-3-linear.json'
    'config/4x4-4inear.json'
    'config/4x4-linear.json'
    'config/4x4-2-linear.json'
    'config/4x4-3-linear.json'
    'config/4x4-4-linear.json'
    'config/4x4-linear.json'
    'config/4x4-2-linear.json'
    'config/4x4-3-linear.json'
    'config/4x4-4-linear.json'
    'config/4x4-linear.json'
    'config/4x4-2-linear.json'
    'config/4x4-3-linear.json'
    'config/4x4-4-linear.json'
    'config/4x4-linear.json'
    'config/4x4-2-linear.json'
    'config/4x4-3-linear.json'
    'config/4x4-4-linear.json'
    'config/4x4-linear.json'
    'config/4x4-2-linear.json'
    'config/4x4-3-linear.json'
    'config/4x4-4-linear.json'
    'config/4x5-linear.json'
    'config/4x5-2-linear.json')

echo "Compiling inqcc..."
cargo build --release

if [ $? -ne 0 ]; then
   echo 'Failed to build inqcc.'
   exit 1
fi

echo "Creating a directory ${outdir} for outputs..."
mkdir -p $outdir

for ((i=0; i<${#targets[@]}; i++))
do
    run_qasm ${targets[i]} ${configs[i]}
done
