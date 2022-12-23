pub mod router;
pub mod telegate_only;
pub mod teledata_only;

pub use router::{RemoteOp, RemoteOpRouter};
pub use telegate_only::*;
pub use teledata_only::*;

#[derive(Debug, Clone, clap::ArgEnum)]
pub enum Strategy {
    TelegateOnly,
    TeledataOnly,
}
