use clap::Clap;
use std::path::PathBuf;

mod dump;

#[derive(Debug, Clap)]
#[clap(author = "Charles \"Demurgos\" Samborski")]
struct CliArgs {
  /// Report messages using JSON.
  #[clap(long = "json")]
  json: bool,
  #[clap(subcommand)]
  command: CliCommand,
}

#[derive(Debug, Clap)]
enum CliCommand {
  /// Extract all data from SWF file.
  #[clap(name = "dump")]
  Dump(DumpArgs),
}

/// A subcommand for controlling testing
#[derive(Debug, Clap)]
struct DumpArgs {
  /// Input SWF file.
  swf: PathBuf,
  /// Output directory.
  output: PathBuf,
}

fn main() {
  let args: CliArgs = CliArgs::parse();

  match &args.command {
    CliCommand::Dump(ref dump_args) => {
      dump_cmd(dump_args);
    }
  };
}

fn dump_cmd(args: &DumpArgs) {
  let swf_file = std::fs::File::open(&args.swf).expect("Failed to open SWF file");
  let mut reader = std::io::BufReader::new(swf_file);

  dump::dump(&args.output, &mut reader);
}
