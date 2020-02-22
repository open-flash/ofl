use crate::dump::{find_avm1, Avm1Location};
use avm1_parser::parse_cfg;
use avm1_types::cfg::Cfg;
use clap::Clap;
use serde::ser::Serialize;
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use swf_parser::parse_swf;
use swf_parser::streaming::movie::parse_swf_signature;

mod dump;
mod report;

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
  /// Extract all data from a SWF file.
  #[clap(name = "dump")]
  Dump(DumpArgs),

  /// Parse a SWF file.
  #[clap(name = "parse")]
  Parse(ParseArgs),
}

/// Arguments to the `dump` subcommand.
#[derive(Debug, Clap)]
struct DumpArgs {
  /// Force the creation of the output directory and ensure it is empty.
  ///
  /// It will remove any existing content at the output path and create an empty directory.
  /// The parent directories will be created automatically if they are missing.
  /// This is roughly equivalent to running the following Shell command:
  /// ```
  /// rm --recursive --force "${OUTPUT}" && mkdir --parents "${OUTPUT}"
  /// ```
  #[clap(long = "force")]
  force: bool,
  /// Input SWF file.
  swf: PathBuf,
  /// Output directory.
  output: Option<PathBuf>,
}

/// Arguments to the `parse` subcommand.
#[derive(Debug, Clap)]
struct ParseArgs {
  /// Input SWF file.
  swf: PathBuf,
}

#[tokio::main]
async fn main() {
  let args: CliArgs = CliArgs::parse();

  let code = match &args.command {
    CliCommand::Dump(ref dump_args) => dump_cmd(dump_args).await,
    CliCommand::Parse(ref parse_args) => parse_cmd(parse_args).await,
  };

  std::process::exit(code);
}

async fn parse_cmd(args: &ParseArgs) -> i32 {
  let swf_bytes = match tokio::fs::read(&args.swf).await {
    Ok(bytes) => bytes,
    Err(e) => {
      eprintln!("Failed to read input SWF");
      eprintln!("{:?}", &e);
      return exitcode::NOINPUT;
    }
  };

  let movie = match parse_swf(&swf_bytes) {
    Ok(movie) => movie,
    Err(e) => {
      eprintln!("Failed to parse SWF file. Please report this error at https://github.com/open-flash/swf-parser/");
      eprintln!("{:?}", &e);
      return exitcode::DATAERR;
    }
  };

  {
    let stdout = std::io::stdout();
    let stdout_lock = stdout.lock();
    let mut ser = serde_json_v8::Serializer::pretty(stdout_lock);
    movie.serialize(&mut ser).expect("Failed to serialize movie");
    ser.into_inner().write_all(b"\n").expect("Failed to write movie");
  }
  exitcode::OK
}

async fn dump_cmd(args: &DumpArgs) -> i32 {
  eprintln!("Step 0: Initialization");
  let cwd = match std::env::current_dir() {
    Ok(cwd) => cwd,
    Err(e) => {
      eprintln!("Failed to resolve current working directory:");
      eprintln!("{:?}", &e);
      return exitcode::IOERR;
    }
  };

  eprintln!("Current Working Directory: {}", cwd.display());
  let swf_path = &args.swf;
  eprintln!("SWF path: {}", swf_path.display());
  let output_dir_path = match &args.output {
    Some(output_dir) => output_dir.to_path_buf(),
    None => {
      dbg!(swf_path);
      dbg!(swf_path.parent());
      let dir_name: OsString = match swf_path.file_stem() {
        Some(stem) => stem.to_os_string(),
        None => OsString::from("ofl-dump"),
      };
      let swf_parent_dir = match swf_path.parent() {
        Some(dir) => dir,
        None => {
          eprintln!("Failed to resolve default output directory");
          return exitcode::USAGE;
        }
      };
      swf_parent_dir.join(dir_name)
    }
  };
  let output_dir_path = &output_dir_path;
  eprintln!("Output directory: {}", output_dir_path.display());
  {
    let output_check = check_output_dir(output_dir_path);
    match output_check {
      OutputDirCheck::PresentEmpty => {} // Perfect case, nothing to do
      OutputDirCheck::PresentNonEmpty => {
        if !args.force {
          eprintln!(
            "Aborting because the output directory is not empty. Use `--force` to empty the directory automatically."
          );
          return exitcode::USAGE;
        }
        eprintln!("The output directory is not empty. Clearing its content because `--force` is enabled.");
        match clean_dir(output_dir_path) {
          Ok(()) => {}
          Err(e) => {
            eprintln!("Failed to clear content of the output directory:");
            eprintln!("{:?}", &e);
            return exitcode::IOERR;
          }
        }
      }
      OutputDirCheck::PresentNonDir => {
        if !args.force {
          eprintln!("Aborting because the output path is not a directory. Use `--force` to overwrite it.");
          return exitcode::USAGE;
        }
        eprintln!(
          "The output path is not a directory, replacing it with an empty directory because `--force` is enabled."
        );
        match remove_all(output_dir_path) {
          Ok(()) => {}
          Err(e) => {
            eprintln!("Failed to remove pre-existing content at the output path");
            eprintln!("{:?}", &e);
            return exitcode::IOERR;
          }
        }
        match fs::create_dir(output_dir_path) {
          Ok(()) => {}
          Err(e) => {
            eprintln!("Failed to create output directory");
            eprintln!("{:?}", &e);
            return exitcode::IOERR;
          }
        }
      }
      OutputDirCheck::MissingSelf => match fs::create_dir(output_dir_path) {
        Ok(()) => {}
        Err(e) => {
          eprintln!("Failed to create output directory");
          eprintln!("{:?}", &e);
          return exitcode::IOERR;
        }
      },
      OutputDirCheck::MissingParent => {
        if !args.force {
          eprintln!("Aborting because the parents of the output directory do not exist. Use `--force` to create them automatically.");
          return exitcode::USAGE;
        }
        match fs::create_dir_all(output_dir_path) {
          Ok(()) => {}
          Err(e) => {
            eprintln!("Failed to create output directory");
            eprintln!("{:?}", &e);
            return exitcode::IOERR;
          }
        }
      }
    }
  }

  eprintln!("Step 1: Read SWF file");

  let swf_bytes = match tokio::fs::read(&args.swf).await {
    Ok(bytes) => bytes,
    Err(e) => {
      eprintln!("Failed to read input SWF");
      eprintln!("{:?}", &e);
      return exitcode::NOINPUT;
    }
  };

  eprintln!("File size (bytes): {}", swf_bytes.len());
  eprintln!("SHA-256: {}", hex::encode(Sha256::digest(&swf_bytes)));

  let swf_signature = match parse_swf_signature(&swf_bytes) {
    Ok((_, signature)) => signature,
    Err(e) => {
      eprintln!("Invalid SWF signature. The file is corrupted or not an SWF file.");
      eprintln!("{:?}", &e);
      return exitcode::DATAERR;
    }
  };

  eprintln!("Compression method: {:?}", swf_signature.compression_method);
  eprintln!("Uncompressed size (bytes): {}", swf_signature.uncompressed_file_length);

  eprintln!("Step 2: Analyze SWF file");
  let movie = match parse_swf(&swf_bytes) {
    Ok(movie) => movie,
    Err(e) => {
      eprintln!("Failed to parse SWF file. Please report this error at https://github.com/open-flash/swf-parser/");
      eprintln!("{:?}", &e);
      return exitcode::DATAERR;
    }
  };

  eprintln!("Saving parsed movie into output directory");
  dump::dump_movie(output_dir_path, &movie);

  eprintln!("Unimplemented: Display stats about the number of tags and their type (definition, action, etc.)");

  eprintln!("Step 3: Analyze AVM1 bytecode");
  let avm1_buffers = find_avm1(&movie);
  if avm1_buffers.is_empty() {
    eprintln!("No AVM1 buffers found");
  } else {
    eprintln!("AVM1 buffers found: {}", avm1_buffers.len());
  }
  for (loc, avm1_buffer) in avm1_buffers.iter() {
    let (avm1_path, cfg_path) = match loc {
      Avm1Location::RootDoAction { tag_index } => {
        let dir = output_dir_path.join(format!("{}", tag_index));
        (dir.join("main.avm1"), dir.join("main.cfg.json"))
      }
      Avm1Location::RootDoInitAction { tag_index } => {
        let dir = output_dir_path.join(format!("{}", tag_index));
        (dir.join("main.avm1"), dir.join("main.cfg.json"))
      }
      Avm1Location::SpriteDoAction {
        tag_index,
        sprite_tag_index,
      } => {
        let dir = output_dir_path
          .join(format!("{}", tag_index))
          .join(format!("{}", sprite_tag_index));
        (dir.join("main.avm1"), dir.join("main.cfg.json"))
      }
      Avm1Location::SpriteDoInitAction {
        tag_index,
        sprite_tag_index,
      } => {
        let dir = output_dir_path
          .join(format!("{}", tag_index))
          .join(format!("{}", sprite_tag_index));
        (dir.join("main.avm1"), dir.join("main.cfg.json"))
      }
    };
    {
      let file = std::fs::File::create(avm1_path).expect("Failed to create AVM1 file");
      let mut writer = std::io::BufWriter::new(file);
      writer.write_all(avm1_buffer).expect("Failed to write AVM1");
    }
    let cfg: Cfg = parse_cfg(avm1_buffer);
    {
      let file = std::fs::File::create(cfg_path).expect("Failed to create CFG file");
      let writer = std::io::BufWriter::new(file);
      let mut ser = serde_json_v8::Serializer::pretty(writer);
      cfg.serialize(&mut ser).expect("Failed to serialize CFG");
      ser.into_inner().write_all(b"\n").expect("Failed to write CFG");
    }
  }

  eprintln!("Success: dump complete");

  exitcode::OK
}

enum OutputDirCheck {
  /// The parent directory does not exist
  MissingParent,
  /// The parent directory exists but the check directory does not
  MissingSelf,
  /// The path exists but the entry is not a directory
  PresentNonDir,
  /// The directory exists and is non-empty
  PresentNonEmpty,
  /// The directory exists and is empty
  PresentEmpty,
}

fn check_output_dir(path: impl AsRef<Path>) -> OutputDirCheck {
  let path = path.as_ref();
  let err = match path.read_dir() {
    Ok(mut entries) => {
      return if entries.next().is_none() {
        OutputDirCheck::PresentEmpty
      } else {
        OutputDirCheck::PresentNonEmpty
      }
    }
    Err(e) => e,
  };
  let err = match err.kind() {
    std::io::ErrorKind::NotFound => {
      let parent_path = match path.parent() {
        None => return OutputDirCheck::MissingParent,
        Some(p) => p,
      };
      return match exists(parent_path) {
        Ok(true) => OutputDirCheck::MissingSelf,
        _ => OutputDirCheck::MissingParent,
      };
    }
    std::io::ErrorKind::Other => {
      //  On Linux, if the path exists but isn't a directory the error is:
      // `Os { code: 20, kind: Other, message: "Not a directory" }`
      match path.symlink_metadata() {
        Ok(metadata) => {
          if !metadata.is_dir() {
            return OutputDirCheck::PresentNonDir;
          } else {
            err
          }
        }
        Err(e) => e,
      }
    }
    _ => err,
  };
  panic!("Failed directory check: {}: {:?}", path.display(), err)
}

//fn is_empty_dir(path: impl AsRef<Path>) -> Result<bool, std::io::Error> {
//  let mut entries = path.as_ref().read_dir()?;
//  Ok(entries.next().is_none())
//}

fn exists(path: impl AsRef<Path>) -> Result<bool, std::io::Error> {
  match path.as_ref().metadata() {
    Ok(_) => Ok(true),
    Err(e) => match e.kind() {
      std::io::ErrorKind::NotFound => Ok(false),
      _ => Err(e),
    },
  }
}

/// Removes the content of a directory (but not the directory itself)
fn clean_dir(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
  for entry in fs::read_dir(path)? {
    let entry = entry?;
    let path = entry.path();
    let file_type = entry.file_type()?;
    remove_all_with_type(&path, file_type)?;
  }
  Ok(())
}

/// Like `std::fs::remove_dir_all`, but for any file file type.
fn remove_all(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
  let path = path.as_ref();
  let metadata = path.metadata()?;
  let file_type = metadata.file_type();
  remove_all_with_type(path, file_type)
}

fn remove_all_with_type(path: &Path, file_type: fs::FileType) -> Result<(), std::io::Error> {
  if file_type.is_dir() {
    fs::remove_dir_all(path)
  } else {
    let is_symlink_dir = if file_type.is_symlink() {
      path.symlink_metadata()?.is_dir()
    } else {
      false
    };
    if is_symlink_dir {
      fs::remove_dir(path)
    } else {
      fs::remove_file(path)
    }
  }
}
