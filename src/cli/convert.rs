#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_convert(args: &ConvertArgs) -> Result<()> {
    let doc = OscalDocument::from_file(&args.file)?;
    let target = args.to.to_file_format();
    let output = convert_document(&doc, target, args.output.as_deref())?;
    if args.output.is_none() {
        print!("{output}");
        if !output.ends_with('\n') {
            println!();
        }
    } else if let Some(out_p) = &args.output {
        println!(
            "Converted {} to {} -> {}",
            doc.kind.name(),
            target.extension(),
            out_p.display()
        );
    }
    Ok(())
}
