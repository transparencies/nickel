use std::{
    fs, io,
    path::{self, PathBuf},
    thread,
};

use anyhow::Result;

use git_version::git_version;
use log::debug;
use lsp_server::Connection;

mod actions;
mod analysis;
mod background;
mod codespan_lsp;
mod command;
mod config;
mod contracts;
mod diagnostic;
mod error;
mod field_walker;
mod files;
mod identifier;
mod incomplete;
mod position;
mod requests;
mod server;
mod task_queue;
use server::Server;
mod term;
mod trace;
mod usage;
mod world;

// Default stack size is 1MB on Windows, which is too small. We make it 8MB, which is the default
// size on Linux.
const STACK_SIZE: usize = 8 * 1024 * 1024;

use crate::{config::LspConfig, trace::Trace};

#[derive(clap::Parser, Debug)]
/// The language server of the Nickel language.
#[command(
    author,
    about,
    long_about = None,
    version = format!(
        "{} {} (rev {})",
        env!("CARGO_BIN_NAME"),
        env!("CARGO_PKG_VERSION"),
        // 7 is the length of self.shortRev. the string is padded out so it can
        // be searched for in the binary
        // The crate published on cargo doesn't have the git version, so we use "cargorel" as a
        // fallback value
        git_version!(fallback = &option_env!("NICKEL_NIX_BUILD_REV").unwrap_or("cargorel")[0..7])
    )
)]
struct Options {
    /// The trace output file, disables tracing if not given
    #[arg(short, long)]
    trace: Option<PathBuf>,

    /// If set, this process runs a background evaluation job instead of setting up a language server.
    #[arg(long)]
    background_eval: bool,
}

fn main() -> Result<()> {
    let handle = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    handle.join().unwrap()
}

fn run() -> Result<()> {
    use clap::Parser;

    env_logger::init();

    let options = Options::parse();

    if options.background_eval {
        return background::worker_main();
    }

    if let Some(file) = options.trace {
        let absolute_path = path::absolute(&file)?;
        debug!("Writing trace to {absolute_path:?}");
        Trace::set_writer(csv::Writer::from_writer(io::BufWriter::new(
            fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(file)
                .map_err(|e| anyhow::anyhow!("Failed to open trace file {absolute_path:?}: {e}"))?,
        )))?;
    }

    let connection = buffered_lsp_connection();

    let capabilities = Server::capabilities();

    let initialize_params = connection.initialize(serde_json::to_value(capabilities)?)?;

    debug!("Raw InitializeParams: {initialize_params:?}");
    let config = match initialize_params.get("initializationOptions") {
        Some(opts) => serde_json::from_value::<LspConfig>(opts.clone())?,
        None => LspConfig::default(),
    };

    debug!("Parsed InitializeParams: {config:?}");

    let _server = Server::new(connection, config).run();

    Ok(())
}

/// Creates an LSP connection reading from stdio with an additional message buffer.
fn buffered_lsp_connection() -> Connection {
    // We add an additional buffer for the LSP connection to avoid a race condition that made
    // request cancellation ineffective.
    //
    // The lsp server connection this receives on is a zero size bounded channel. This
    // causes an issue where once we pull something from the channel, the lsp server thread
    // must fetch the next message from stdin, leaving this channel empty for a short time
    // even if there are multiple messages already sent to the server. When it's queueing
    // tasks this loop will usually beat the thread reading from stdin and will find the
    // channel empty. This can cause the main loop to begin handling a request even if
    // there's already a notification in stdin cancelling it.
    //
    // The buffer lets multiple messages stack up so there's not a problem with empty reads from
    // the channel.

    let (buf_sender, buf_receiver) = crossbeam::channel::bounded(30);

    let (Connection { sender, receiver }, _threads) = Connection::stdio();
    std::thread::Builder::new()
        .name("LspMessageBuffer".into())
        .spawn(move || {
            receiver
                .into_iter()
                .try_for_each(|msg| buf_sender.send(msg))
        })
        .unwrap();

    Connection {
        sender,
        receiver: buf_receiver,
    }
}
