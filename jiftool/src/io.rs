use anyhow::Context;
use jif::{Jif, JifRaw};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use tracer_format::{read_deduped_trace, TimestampedAccess};

pub(crate) fn read_jif(path: &Path) -> anyhow::Result<Jif> {
    let mut file = BufReader::new(
        File::open(path).with_context(|| format!("failed to open JIF {}", path.display()))?,
    );
    Jif::from_reader(&mut file).with_context(|| format!("failed to read JIF {}", path.display()))
}

pub(crate) fn read_raw_jif(path: &Path) -> anyhow::Result<JifRaw> {
    let mut file = BufReader::new(
        File::open(path).with_context(|| format!("failed to open JIF {}", path.display()))?,
    );
    JifRaw::from_reader(&mut file)
        .with_context(|| format!("failed to read raw JIF {}", path.display()))
}

pub(crate) fn write_jif(path: &Path, jif: &mut Jif, show: bool) -> anyhow::Result<()> {
    let mut output = BufWriter::new(
        File::create(path).with_context(|| format!("failed to create JIF {}", path.display()))?,
    );
    let raw = JifRaw::from_materialized_ref(jif);

    if show {
        println!("{raw:#x?}");
    }

    raw.to_writer(&mut output)
        .with_context(|| format!("failed to write JIF {}", path.display()))?;
    Ok(())
}

pub(crate) fn read_trace_file(path: &Path) -> anyhow::Result<Vec<TimestampedAccess>> {
    let file = BufReader::new(
        File::open(path).with_context(|| format!("failed to open trace {}", path.display()))?,
    );
    read_deduped_trace(file).with_context(|| format!("failed to read trace {}", path.display()))
}

pub(crate) fn read_trace_input(path: Option<&Path>) -> anyhow::Result<Vec<TimestampedAccess>> {
    match path {
        Some(path) => read_trace_file(path),
        None => {
            let stdin = std::io::stdin();
            read_deduped_trace(stdin.lock()).context("failed to read trace from stdin")
        }
    }
}
