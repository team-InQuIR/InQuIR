#!/bin/bash

outdir=results


run_qasm16() {
    local targets=(
        'benchmark/quantum_compiler_optim/examples/adr4_197.qasm'
        'benchmark/quantum_compiler_optim/examples/4gt12-v1_89.qasm'
        'benchmark/quantum_compiler_optim/examples/9symml_195.qasm'
        'benchmark/quantum_compiler_optim/examples/ising_model_16.qasm'
        'benchmark/quantum_compiler_optim/examples/life_238.qasm'
        'benchmark/quantum_compiler_optim/examples/root_255.qasm'
        'benchmark/quantum_compiler_optim/examples/rd53_138.qasm'
        'benchmark/quantum_compiler_optim/examples/sqn_258.qasm')
    local configs=(
        'config/2x8x1-linear.json'
        'config/2x8x2-linear.json'
        'config/2x8x4-linear.json'
        'config/2x8-cube.json'
	      'config/2x9-torus.json')


    for ((j=0; j<${#targets[@]}; j++))
    do
	local target=${targets[j]}
        local base=`basename ${target%.qasm}`
        for ((k=0; k<${#configs[@]}; k++))
        do
	          local config=${configs[k]}
	          local config_name=`basename ${config%.json}`
            local st='telegate-only'
            local common_name="${base}_${st}_${config_name}"
            local out="${common_name}.inq"
            local met="${common_name}.json"
            local timestamp="${common_name}.timestamp"
            echo "Compiling ${target} for ${out}"
            ./target/release/inqcc ${target} \
                                   -o ${outdir}/${out} \
                                   --config ${config} \
                                   --strategy ${st} \
                                   --metrics ${outdir}/${met} \
                                   --timestamp ${outdir}/${timestamp}

            echo 'Plot timestamps...'
            python3 ./scripts/plot_timestamp.py ${outdir}/${timestamp} \
                    -o ${outdir}/${common_name}.pdf
        done
    done
}

echo "Compiling inqcc..."
cargo build --release

if [ $? -ne 0 ]; then
   echo 'Failed to build inqcc.'
   exit 1
fi

echo "Creating a directory ${outdir} for outputs..."
mkdir -p $outdir

run_qasm16
