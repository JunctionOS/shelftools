use crate::{io, tsa};
use anyhow::Context;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub(crate) struct ModifyArgs {
    #[arg(value_name = "INPUT", value_hint = clap::ValueHint::FilePath)]
    input: PathBuf,

    #[arg(value_name = "OUTPUT", value_hint = clap::ValueHint::FilePath)]
    output: PathBuf,

    #[arg(long)]
    show: bool,

    #[command(subcommand)]
    operation: ModifyOperation,
}

#[derive(Subcommand)]
enum ModifyOperation {
    /// Rewrite the JIF without semantic changes.
    Rewrite,

    /// Rename a referenced file path.
    Rename { old_path: String, new_path: String },

    /// Build interval trees.
    BuildItrees {
        #[arg(value_name = "CHROOT", value_hint = clap::ValueHint::DirPath)]
        chroot_path: Option<PathBuf>,
    },

    /// Fragment VMAs while still finding zero pages and reference segments.
    FragmentVmas {
        #[arg(value_name = "CHROOT", value_hint = clap::ValueHint::DirPath)]
        chroot_path: Option<PathBuf>,
    },

    /// Set up the prefetch section.
    SetupPrefetch,

    /// Mark VMAs that are referenced by the ordering section.
    TagVmas,

    /// Add an ordering section from a timestamped access log.
    AddOrd {
        #[arg(value_name = "TRACE", value_hint = clap::ValueHint::FilePath)]
        time_log: Option<PathBuf>,
    },
}

pub(crate) fn run(args: ModifyArgs) -> anyhow::Result<()> {
    let mut jif = io::read_jif(&args.input)?;

    match args.operation {
        ModifyOperation::Rewrite => {}
        ModifyOperation::Rename { old_path, new_path } => jif.rename_file(&old_path, &new_path),
        ModifyOperation::BuildItrees { chroot_path } => jif
            .build_itrees(chroot_path)
            .context("failed to build interval trees")?,
        ModifyOperation::FragmentVmas { chroot_path } => jif
            .fragment_vmas(chroot_path)
            .context("failed to fragment VMAs")?,
        ModifyOperation::SetupPrefetch => {
            jif.setup_prefetch().context("failed to set up prefetch")?
        }
        ModifyOperation::TagVmas => jif.tag_vmas(),
        ModifyOperation::AddOrd { time_log } => {
            let trace = io::read_trace_input(time_log.as_deref())?;
            let ords = tsa::construct_ord_chunks(&jif, trace);
            jif.add_ordering_info(ords)?;
        }
    }

    io::write_jif(&args.output, &mut jif, args.show)
}
