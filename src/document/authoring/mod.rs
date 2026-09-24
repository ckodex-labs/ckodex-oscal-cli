pub mod assembler;
pub mod splitter;
pub mod templates;

pub use assembler::{AssembleReport, assemble_directory};
pub use splitter::{SplitReport, split_document};
pub use templates::{TemplateReport, scaffold_template};
