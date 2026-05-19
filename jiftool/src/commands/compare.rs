use crate::digest::{digest_jif, print_intersections, write_upset_input, DigestOptions, JifDigest};
use crate::{io, python};
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

const PLOT_UPSET_PY: &str = "
import matplotlib.pyplot as plt
import upsetplot
import sys

if __name__ == '__main__':
    data = dict()
    for line in sys.stdin.readlines():
        split_colon = line.strip().split(':')
        assert len(split_colon) == 2, 'expected format is <filename>: [<hashes>, ]'

        filename = split_colon[0]
        hashes = set(a.strip() for a in split_colon[1].strip().split(',') if len(a) > 0)
        data[filename] = hashes

    upset_data = upsetplot.from_contents(data)
    upsetplot.plot(upset_data, show_counts='{:,}')
    plt.suptitle('Intersection of {} regions among jif snapshots'.format(sys.argv[1]))
    plt.savefig(sys.argv[2])
";

#[derive(Args)]
pub(crate) struct CompareArgs {
    #[arg(value_name = "FILE", num_args = 2.., value_hint = clap::ValueHint::FilePath)]
    jif_files: Vec<PathBuf>,

    #[arg(short, long, conflicts_with = "private")]
    shared: bool,

    #[arg(short, long, conflicts_with = "shared")]
    private: bool,

    #[arg(long)]
    ordering: bool,

    #[arg(short, long, value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    output: Option<PathBuf>,
}

pub(crate) fn run(args: CompareArgs) -> anyhow::Result<()> {
    let options = DigestOptions {
        include_private: !args.shared,
        include_shared: !args.private,
        ordering_only: args.ordering,
    };

    let digests = args
        .jif_files
        .into_iter()
        .map(|path| {
            let jif = io::read_jif(&path)?;
            Ok::<_, anyhow::Error>((path, digest_jif(&jif, options)))
        })
        .collect::<Result<HashMap<PathBuf, JifDigest>, _>>()?;

    if let Some(output) = args.output {
        let plot_title = if args.shared {
            "shared"
        } else if args.private {
            "private"
        } else {
            "all"
        };

        python::run_python(
            PLOT_UPSET_PY,
            &[plot_title.to_string(), output.display().to_string()],
            "matplotlib and upsetplot",
            |stdin| write_upset_input(digests, stdin),
        )?;
    } else {
        print_intersections(&digests);
    }

    Ok(())
}
