use clap::{Parser, Subcommand};

mod commands;
mod digest;
mod io;
mod python;
mod tsa;

#[derive(Parser)]
#[command(version, about = "Read, modify, compare, and trace JIF files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read or inspect a JIF file.
    Read(commands::read::ReadArgs),

    /// Check that a JIF file parses.
    Check(commands::check::CheckArgs),

    /// Write a modified copy of a JIF file.
    Modify(commands::modify::ModifyArgs),

    /// Add JIF context to a timestamped access trace.
    Trace(commands::trace::TraceArgs),

    /// Compare page identities across JIF files.
    Compare(commands::compare::CompareArgs),

    /// Plot first-access timing from a trace.
    Time(commands::time::TimeArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Read(args) => commands::read::run(args),
        Command::Check(args) => commands::check::run(args),
        Command::Modify(args) => commands::modify::run(args),
        Command::Trace(args) => commands::trace::run(args),
        Command::Compare(args) => commands::compare::run(args),
        Command::Time(args) => commands::time::run(args),
    }
}
