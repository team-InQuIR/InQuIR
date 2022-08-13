use inqcc::{
    arch::Configuration,
    parser::qasm2,
    codegen::codegen,
    codegen::allocation::{NaiveNodeAllocator, NodeAllocator},
    codegen::always_rcx::AlwaysRemoteAllocator,
};
use inquir::metrics::Metrics;

use std::fs;
use std::path::Path;
use std::io::Write;
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

    /// Path to the output file
    #[clap(short, long)]
    output: Option<String>,

    /// Path to the configuration file
    #[clap(short, long)]
    config: String,

    /// Compilation strategy
    #[clap(arg_enum, long)]
    strategy: Strategy,
}

fn output_to_inquir_file(filename: String, program: &Vec<Vec<inquir::Expr>>) -> Result<(), std::io::Error> {
    let mut file = fs::File::create(filename)?;
    for (node, exps) in program.iter().enumerate() {
        writeln!(file, "node {}:", node)?;
        for e in exps {
            writeln!(file, "  {}", e)?;
        }
        writeln!(file, "end.\n")?;
    }
    Ok(())
}

fn main() {
    let args = Args::parse();
    println!("config: {}", args.config);
    let config = Configuration::from_json(args.config);
    println!("{:?}", config);

    let source = fs::read_to_string(&args.input).unwrap();
    let hir_exps = qasm2::parse(&source).unwrap();
    //println!("Finished parse.");

    let allocator: Box<dyn NodeAllocator> = match args.strategy {
        Strategy::AlwaysMove => Box::new(NaiveNodeAllocator::new(&hir_exps, &config)),
        Strategy::AlwaysRemote => Box::new(AlwaysRemoteAllocator::new(&hir_exps, &config)),
    };

    let res = codegen(hir_exps, &config, allocator);
    let output_filename = if let Some(filename) = args.output {
        filename
    } else {
        Path::new(&args.input).file_stem().unwrap().to_str().unwrap().to_owned() + ".inq"
    };
    output_to_inquir_file(output_filename, &res).unwrap();

    let metrics = Metrics::new(&res);
    println!("Metrics:");
    println!("  Communication depth: {}", metrics.comm_depth());
}
