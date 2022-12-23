use inqcc::{
    arch::Configuration,
    parser::qasm2,
    codegen::codegen,
    codegen::routing::{Strategy, RemoteOpRouter, TelegateOnly, TeledataOnly},
    metrics::Metrics,
    dependency_graph::DependencyGraphBuilder,
    simulation::simulator::Simulator,
};
use inquir::System;

use std::fs;
use std::path::Path;
use std::io::Write;
use clap::Parser;


#[derive(Parser, Debug)]
#[clap(author, version = "0.0.0", about, long_about = None)]
struct Args {
    /// Path to the QASM file
    input: String,

    /// Path to the output file
    #[clap(short, long)]
    output: Option<String>,

    /// Path to the configuration file
    #[clap(short, long)]
    config: String,

    /// Compilation strategy
    #[clap(arg_enum, long)]
    strategy: Strategy,

    #[clap(long)]
    metrics: Option<String>,

    /// Enable quasi-parallelism.
    #[clap(long)]
    quasi_para: bool,

    /// Where a dependency graph is output.
    #[clap(long)]
    depends: Option<String>,
}

fn output_to_inquir_file(filename: &String, program: &System) -> Result<(), std::io::Error> {
    let mut file = fs::File::create(filename)?;
    write!(file, "{}", program)
}

fn output_metrics(filename: &String, metrics: &Metrics) -> Result<(), std::io::Error> {
    let serialized = serde_json::to_string(&metrics).unwrap();
    let mut file = fs::File::create(filename)?;
    write!(file, "{}", serialized)
}

fn main() {
    let args = Args::parse();
    println!("config: {}", args.config);
    let config = Configuration::from_json(args.config);
    //println!("{:?}", config);

    let source = fs::read_to_string(&args.input).unwrap();
    let hir_exps = qasm2::parse(&source).unwrap();
    //println!("Finished parse.");

    let router: Box<dyn RemoteOpRouter> = match args.strategy {
        Strategy::TeledataOnly => Box::new(TeledataOnly::new(&hir_exps, &config)),
        Strategy::TelegateOnly => Box::new(TelegateOnly::new(&hir_exps, &config)),
    };

    let res = codegen(hir_exps, &config, router, args.quasi_para);
    let output_filename = if let Some(filename) = args.output {
        filename
    } else {
        Path::new(&args.input).file_stem().unwrap().to_str().unwrap().to_owned() + ".inq"
    };
    output_to_inquir_file(&output_filename, &res).unwrap();
    if let Some(depends_path) = args.depends {
        let mut file = std::fs::File::create(depends_path).unwrap();
        let builder = DependencyGraphBuilder::new();
        let graphviz = builder.build(res.clone()).as_graphviz();
        write!(file, "{}", graphviz).unwrap();
    };

    if let Some(met_path) = args.metrics {
        let simulator = Simulator::new(&res, &config);
        let cost = simulator.run();
        let metrics = Metrics::new(&res, cost);
        println!("Metrics:");
        println!("  E-depth: {}", metrics.e_depth());
        println!("  E-count: {}", metrics.e_count());
        println!("  C-depth: {}", metrics.c_depth());
        println!("  C-count: {}", metrics.c_count());
        println!("  Execution cost: {}", metrics.time());
        output_metrics(&met_path, &metrics).unwrap();
    }
}
