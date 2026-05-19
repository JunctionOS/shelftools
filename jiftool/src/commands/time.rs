use crate::{io, python};
use clap::Args;
use std::path::PathBuf;

const PLOT_TIME_PY: &str = "
import matplotlib.pyplot as plt
import sys

if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit('usage: time plotter <output filename> <plot title>')

    output = '{}.pdf'.format(sys.argv[1])
    title = sys.argv[2]

    all_x = []
    all_y = []
    non_shared_x = []
    non_shared_y = []
    private_x = []
    private_y = []

    private_cnt = 0
    zero_cnt = 0
    shared_cnt = 0
    for line in sys.stdin.readlines():
        timestamp_ms, source = line.strip().split(' ')
        timestamp_ms = float(timestamp_ms)

        all_x.append(timestamp_ms)
        all_y.append(len(all_x))
        if source == 'private':
            non_shared_x.append(timestamp_ms)
            non_shared_y.append(len(non_shared_x))
            private_x.append(timestamp_ms)
            private_y.append(len(private_x))
            private_cnt += 1
        elif source == 'zero':
            non_shared_x.append(timestamp_ms)
            non_shared_y.append(len(non_shared_x))
            zero_cnt += 1
        elif source == 'shared':
            shared_cnt += 1

    plt.scatter(all_x, all_y, s=5, label='all')
    plt.scatter(non_shared_x, non_shared_y, s=5, label='private')
    plt.scatter(private_x, private_y, s=5, label='private - zero')
    plt.xlabel('Time (ms)', fontfamily='sans-serif', fontsize=12)
    plt.ylabel('Number of unique pages', fontfamily='sans-serif', fontsize=12)
    plt.title(title, fontfamily='sans-serif', fontsize=15)
    plt.legend()
    plt.savefig(output)
    print('{}, \\t{}, \\t{}, \\t{}, \\t{}'.format(title, len(all_x), private_cnt, shared_cnt, zero_cnt))
";

#[derive(Args)]
pub(crate) struct TimeArgs {
    #[arg(value_name = "JIF", value_hint = clap::ValueHint::FilePath)]
    jif_file: PathBuf,

    #[arg(value_name = "TRACE", value_hint = clap::ValueHint::FilePath)]
    trace_file: PathBuf,

    #[arg(value_name = "OUTPUT", value_hint = clap::ValueHint::FilePath)]
    output_file: PathBuf,

    #[arg(long)]
    title: Option<String>,
}

pub(crate) fn run(args: TimeArgs) -> anyhow::Result<()> {
    let jif = io::read_jif(&args.jif_file)?;
    let trace = io::read_trace_file(&args.trace_file)?;
    let title = args.title.unwrap_or_else(|| {
        args.trace_file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("<default>")
            .to_string()
    });

    let stdout = python::run_python(
        PLOT_TIME_PY,
        &[args.output_file.display().to_string(), title],
        "matplotlib",
        |stdin| {
            for entry in &trace {
                let timestamp_ms = entry.usecs as f64 / 1000.0;
                let source = jif
                    .source_at(entry.masked_addr() as u64)
                    .map(|source| source.as_str())
                    .unwrap_or("unknown");
                writeln!(stdin, "{timestamp_ms} {source}")?;
            }
            Ok(())
        },
    )?;

    print!("{stdout}");
    Ok(())
}
