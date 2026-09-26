pub mod constants;
pub mod csv_io;
pub mod error;
pub mod graph;
pub mod graph_output;
pub mod models;
pub mod records;
pub mod setup;
pub mod traits;

pub use csv_io::{find_atk_param_pc, find_bullet, find_magic};
pub use error::AppError;
pub use graph::MagicGraph;
pub use records::SearchField;
pub use setup::setup_csv;

pub fn build_magic_graph(search_field: SearchField) -> Result<MagicGraph, AppError> {
    let magic = find_magic(&search_field)?;

    MagicGraph::build(magic.id)
}
