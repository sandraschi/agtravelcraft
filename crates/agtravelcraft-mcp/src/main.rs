#![forbid(unsafe_code)]

mod mcp;

use std::path::PathBuf;
use std::process::ExitCode;

use agtravelcraft_core::bus::Command;
use agtravelcraft_core::default_db_path;
use agtravelcraft_core::store::TravelStore;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "agtravelcraft-mcp",
    version,
    about = "AgTravelCraft CLI and MCP 2026-07-28 stdio server"
)]
struct Cli {
    #[arg(long, global = true)]
    db: Option<PathBuf>,

    /// Emit machine-readable JSON output (default format).
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: SubCmd,
}

#[derive(Subcommand, Debug)]
enum SubCmd {
    /// Run a full Vienna -> Zurich -> Osaka -> Tokyo -> Vienna Customs demo voyage.
    Demo,
    /// Print database and customs status.
    Status,
    /// List recorded diplomatic voyages.
    Voyages {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// List quarantined Vienna Customs records.
    Customs {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Human-only Vienna Customs approval or rejection.
    Decide {
        #[arg(long)]
        voyage_id: String,
        #[arg(long)]
        approve: bool,
        #[arg(long)]
        reviewer: String,
    },
    /// Human-only selective turn redaction at Vienna Customs.
    Redact {
        #[arg(long)]
        voyage_id: String,
        #[arg(long)]
        turn: usize,
        #[arg(long)]
        reviewer: String,
    },
    /// Generate Resonite or Overte Diplomatic Teahouse 3D stage script.
    Stage {
        #[arg(long)]
        voyage_id: String,
        #[arg(long, default_value = "resonite")]
        target: String,
    },
    /// Export Storytold-ready Markdown chronicle for a voyage.
    Chronicle {
        #[arg(long)]
        voyage_id: String,
    },
    /// Create an atomic SQLite VACUUM INTO backup snapshot.
    Backup {
        #[arg(long)]
        dest: String,
    },
    /// List bundled workflow skills.
    Skills,
    /// Run MCP 2026-07-28 stdio server.
    Mcp,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db_path);
    let store = match TravelStore::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("agtravelcraft-mcp: failed to open database: {e}");
            return ExitCode::FAILURE;
        }
    };

    match cli.command {
        SubCmd::Mcp => match mcp::run_stdio(&store) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("agtravelcraft-mcp stdio error: {e}");
                ExitCode::FAILURE
            }
        },
        SubCmd::Demo => run_and_print(Command::SeedDemo, &store),
        SubCmd::Status => run_and_print(Command::Status, &store),
        SubCmd::Voyages { limit } => {
            run_and_print(Command::ListVoyages { limit: Some(limit) }, &store)
        }
        SubCmd::Customs { limit } => {
            run_and_print(Command::ListCustoms { limit: Some(limit) }, &store)
        }
        SubCmd::Decide {
            voyage_id,
            approve,
            reviewer,
        } => run_and_print(
            Command::DecideCustoms {
                voyage_id,
                approve,
                reviewer,
                is_human: true,
            },
            &store,
        ),
        SubCmd::Redact {
            voyage_id,
            turn,
            reviewer,
        } => run_and_print(
            Command::RedactCustomsTurn {
                voyage_id,
                turn_index: turn,
                reviewer,
                is_human: true,
            },
            &store,
        ),
        SubCmd::Stage { voyage_id, target } => {
            run_and_print(Command::BuildStage { voyage_id, target }, &store)
        }
        SubCmd::Chronicle { voyage_id } => {
            run_and_print(Command::ExportChronicle { voyage_id }, &store)
        }
        SubCmd::Backup { dest } => {
            run_and_print(Command::BackupSnapshot { destination: dest }, &store)
        }
        SubCmd::Skills => run_and_print(Command::ListSkills, &store),
    }
}

fn run_and_print(cmd: Command, store: &TravelStore) -> ExitCode {
    match cmd.dispatch(store) {
        Ok(val) => {
            match serde_json::to_string_pretty(&val) {
                Ok(s) => println!("{s}"),
                Err(e) => {
                    eprintln!("serialization error: {e}");
                    return ExitCode::FAILURE;
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
