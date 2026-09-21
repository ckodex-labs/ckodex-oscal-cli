pub mod assembler;
pub mod splitter;
pub mod templates;

pub use assembler::{assemble_directory, AssembleReport};
pub use splitter::{split_document, SplitReport};
pub use templates::{scaffold_template, TemplateReport};
