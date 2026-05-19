use crate::io;
use clap::{Args, Subcommand, ValueEnum};
use jif::{Jif, OrdStats};
use std::path::PathBuf;

#[derive(Args)]
pub(crate) struct ReadArgs {
    #[arg(value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    file: PathBuf,

    #[command(subcommand)]
    command: Option<ReadCommand>,
}

#[derive(Subcommand)]
enum ReadCommand {
    /// Print whole-file counts.
    Summary,

    /// Print materialized pheaders.
    Pheaders(RangeArgs),

    /// Print ordering-section stats or chunks.
    Ord {
        #[arg(long)]
        chunks: bool,
    },

    /// Print raw-format sections.
    Raw {
        #[arg(value_enum, default_value = "summary")]
        section: RawSection,
    },
}

#[derive(Args, Default, Clone, Copy)]
struct RangeArgs {
    #[arg(long, conflicts_with = "index")]
    start: Option<usize>,

    #[arg(long, conflicts_with = "index")]
    end: Option<usize>,

    #[arg(long)]
    index: Option<usize>,
}

#[derive(Clone, Copy, ValueEnum)]
enum RawSection {
    Summary,
    Pheaders,
    Strings,
    Itrees,
    Ord,
    All,
}

pub(crate) fn run(args: ReadArgs) -> anyhow::Result<()> {
    match args.command.unwrap_or(ReadCommand::Summary) {
        ReadCommand::Summary => print_summary(&io::read_jif(&args.file)?),
        ReadCommand::Pheaders(range) => print_pheaders(&io::read_jif(&args.file)?, range),
        ReadCommand::Ord { chunks } => {
            let jif = io::read_jif(&args.file)?;
            if chunks {
                println!("{:#x?}", jif.ord_chunks());
            } else {
                print_ord_stats(jif.ord_stats());
            }
        }
        ReadCommand::Raw { section } => {
            let raw = io::read_raw_jif(&args.file)?;
            match section {
                RawSection::Summary => {
                    println!("pheaders:      {}", raw.pheaders().len());
                    println!("strings:       {}", raw.strings().len());
                    println!("itree nodes:   {}", raw.itree_nodes().len());
                    println!("ord chunks:    {}", raw.ord_chunks().len());
                    println!("metadata size: {:#x} B", raw.data_offset());
                    println!("data size:     {:#x} B", raw.data_size());
                    print_ord_stats(OrdStats::from_chunks(raw.ord_chunks()));
                }
                RawSection::Pheaders => println!("{:#x?}", raw.pheaders()),
                RawSection::Strings => println!("{:#x?}", raw.strings()),
                RawSection::Itrees => println!("{:#x?}", raw.itree_nodes()),
                RawSection::Ord => println!("{:#x?}", raw.ord_chunks()),
                RawSection::All => println!("{raw:#x?}"),
            }
        }
    }

    Ok(())
}

fn selected_slice<T>(items: &[T], range: RangeArgs) -> &[T] {
    if let Some(index) = range.index {
        return items.get(index..index.saturating_add(1)).unwrap_or(&[]);
    }

    let start = range.start.unwrap_or(0).min(items.len());
    let end = range.end.unwrap_or(items.len()).min(items.len());
    if start > end {
        &[]
    } else {
        &items[start..end]
    }
}

fn print_summary(jif: &Jif) {
    let summary = jif.summary();
    println!("pheaders:       {}", summary.pheaders);
    println!("ord chunks:     {}", summary.ord_chunks);
    println!("pages:          {}", summary.pages);
    println!("private pages:  {}", summary.private_pages);
    println!("shared pages:   {}", summary.shared_pages);
    println!("zero pages:     {}", summary.zero_pages);
    println!("intervals:      {}", summary.intervals);
}

fn print_ord_stats(stats: OrdStats) {
    println!("pages:                    {}", stats.pages);
    println!("private pages:            {}", stats.private_pages);
    println!("shared pages:             {}", stats.shared_pages);
    println!("zero pages:               {}", stats.zero_pages);
    println!("written pages:            {}", stats.written_to_pages);
    println!(
        "private written pages:    {}",
        stats.private_written_to_pages
    );
    println!(
        "shared written pages:     {}",
        stats.shared_written_to_pages
    );
    println!("zero written pages:       {}", stats.zero_written_to_pages);
}

fn print_pheaders(jif: &Jif, range: RangeArgs) {
    println!(
        "{:>5} | {:>18} | {:>18} | {:>8} | {:>8} | path",
        "idx", "start", "end", "prot", "pages"
    );

    let pheaders = selected_slice(jif.pheaders(), range);
    let offset = range.index.or(range.start).unwrap_or(0);
    for (idx, pheader) in pheaders.iter().enumerate() {
        let (start, end) = pheader.virtual_range();
        println!(
            "{:>5} | {start:#018x} | {end:#018x} | {:>8} | {:>8} | {}",
            idx + offset,
            format_prot(pheader.prot()),
            pheader.total_pages(),
            pheader.pathname().unwrap_or("<anonymous>"),
        );
    }
}

fn format_prot(prot: u8) -> String {
    format!(
        "{}{}{}{}",
        if prot & jif::Prot::Read as u8 != 0 {
            "r"
        } else {
            "-"
        },
        if prot & jif::Prot::Write as u8 != 0 {
            "w"
        } else {
            "-"
        },
        if prot & jif::Prot::Exec as u8 != 0 {
            "x"
        } else {
            "-"
        },
        if prot & jif::Prot::InOrdering as u8 != 0 {
            "o"
        } else {
            "-"
        },
    )
}
