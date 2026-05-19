use crate::io;
use clap::Args;
use jif::Jif;
use std::path::PathBuf;
use tracer_format::TimestampedAccess;

#[derive(Args)]
pub(crate) struct TraceArgs {
    #[arg(value_name = "JIF", value_hint = clap::ValueHint::FilePath)]
    jif_file: PathBuf,

    #[arg(value_name = "TRACE", value_hint = clap::ValueHint::FilePath)]
    trace_file: PathBuf,
}

pub(crate) fn run(args: TraceArgs) -> anyhow::Result<()> {
    let jif = io::read_jif(&args.jif_file)?;
    let trace = io::read_trace_file(&args.trace_file)?;
    print_trace(&jif, &trace);
    Ok(())
}

fn print_trace(jif: &Jif, trace: &[TimestampedAccess]) {
    for entry in trace {
        let addr = entry.masked_addr() as u64;
        let context = jif.trace_context(addr);
        let source = context
            .source
            .map(|source| source.as_str())
            .unwrap_or("unknown");

        if let Some(pheader) = context.pheader {
            println!(
                "{}: {:#x} | {:#x}-{:#x} | {} | {}",
                entry.usecs,
                context.addr,
                pheader.virtual_range().0,
                pheader.virtual_range().1,
                pheader.pathname().unwrap_or("<unnamed>"),
                source
            );
        } else {
            println!("{}: {:#x} | {}", entry.usecs, context.addr, source);
        }
    }
}
