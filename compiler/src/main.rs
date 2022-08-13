use inqcc::{
    arch::Configuration,
    parser::qasm2,
    codegen::codegen,
    codegen::allocation::{NaiveNodeAllocator, NodeAllocator},
    codegen::always_rcx::AlwaysRemoteAllocator,
};
use inquir::metrics::Metrics;

use std::fs;
use clap::Parser;

#[derive(Debug, Clone, clap::ArgEnum)]
enum Strategy {
    AlwaysMove,
    AlwaysRemote,
}

#[derive(Parser, Debug)]
#[clap(author, version = "0.0.0", about, long_about = None)]
struct Args {
    /// Path to the QASM file
    input: String,

    /// Path to the configuration file
    #[clap(short, long)]
    config: String,

    /// Compilation strategy
    #[clap(arg_enum, short, long)]
    opt: Strategy,
}

fn main() {
    let args = Args::parse();
    println!("config: {}", args.config);
    let config = Configuration::from_json(args.config);
    println!("{:?}", config);

    let source = fs::read_to_string(args.input).unwrap();
    let hir_exps = qasm2::parse(&source).unwrap();
    //println!("Finished parse.");

    let allocator: Box<dyn NodeAllocator> = match args.opt {
        Strategy::AlwaysMove => Box::new(NaiveNodeAllocator::new(&hir_exps, &config)),
        Strategy::AlwaysRemote => Box::new(AlwaysRemoteAllocator::new(&hir_exps, &config)),
    };

    let res = codegen(hir_exps, &config, allocator);
    //println!("Finished code generation.");
    for (node, exps) in res.iter().enumerate() {
        println!("node {}:", node);
        for e in exps {
            println!("  {}", e);
        }
        println!("end.\n");
    }

    let metrics = Metrics::new(&res);
    println!("Metrics:");
    println!("  Communication depth: {}", metrics.comm_depth());
}
