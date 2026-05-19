use crate::io;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub(crate) struct CheckArgs {
    #[arg(value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    file: PathBuf,

    #[arg(long)]
    raw: bool,
}

pub(crate) fn run(args: CheckArgs) -> anyhow::Result<()> {
    if args.raw {
        io::read_raw_jif(&args.file)?;
    } else {
        io::read_jif(&args.file)?;
    }
    Ok(())
}
