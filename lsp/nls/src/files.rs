use std::path::PathBuf;

use anyhow::{Result, anyhow};
use log::info;
use lsp_server::RequestId;
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, Url,
    notification::{DidCloseTextDocument, DidOpenTextDocument, Notification},
};
use nickel_lang_core::files::FileId;

use crate::{
    error::Error,
    task_queue::{DiagnosticsRequest, Priority},
    trace::{Enrich, Trace, param::FileUpdate},
};

use super::server::Server;

pub(crate) fn uri_to_path(uri: &Url) -> std::result::Result<PathBuf, Error> {
    if uri.scheme() != "file" {
        Err(Error::SchemeNotSupported(uri.scheme().into()))
    } else {
        uri.to_file_path()
            .map_err(|_| Error::InvalidPath(uri.clone()))
    }
}

/// Returns a list of open files that were potentially invalidated by the changes.
pub fn handle_open(server: &mut Server, params: DidOpenTextDocumentParams) -> Result<Vec<Url>> {
    let uri = params.text_document.uri;
    let id: RequestId = format!("{}#{}", uri, params.text_document.version).into();

    Trace::receive(id.clone(), DidOpenTextDocument::METHOD);
    Trace::enrich(
        &id,
        FileUpdate {
            content: &params.text_document.text,
        },
    );

    let (file_id, invalid) = server
        .world
        .add_file(uri.clone(), params.text_document.text)?;
    info!("Opened file {uri} with ID {file_id:?}");

    Trace::reply(id);
    Ok(server.world.uris(invalid).cloned().collect())
}

/// handles the textDocument/didClose method.
/// Returns a list of open files that were potentially invalidated by the changes.
pub fn handle_close(
    server: &mut Server,
    params: DidCloseTextDocumentParams,
) -> Result<(Option<FileId>, Vec<Url>)> {
    let uri = params.text_document.uri;
    let id: RequestId = format!("{uri}#close").into();

    Trace::receive(id.clone(), DidCloseTextDocument::METHOD);

    let (new_file_id, invalid) = server.world.close_file(uri.clone())?;
    info!("Closed file {uri}");

    Trace::reply(id);
    Ok((new_file_id, server.world.uris(invalid).cloned().collect()))
}

/// Updates the diagnostics for a file. This will synchronously publish parsing and typechecking
/// diagnostics for the file, as well as creating a background job for evaluation of the file.
pub fn run_diagnostics_on_file(server: &mut Server, req: DiagnosticsRequest) -> Result<()> {
    let file_id = server
        .world
        .file_id(&req.uri)?
        .ok_or_else(|| anyhow!("Could not find a matching File ID for {}", req.uri))?;
    let diags = server.world.parse_and_typecheck(file_id);
    server.issue_diagnostics(file_id, diags);
    match req.priority {
        Priority::High => server
            .background_jobs
            .priority_eval_file(req.uri, &server.world),
        Priority::Normal => server.background_jobs.eval_file(req.uri, &server.world),
    }
    Ok(())
}

/// Returns a list of open files that were potentially invalidated by the changes.
pub fn handle_save(server: &mut Server, params: DidChangeTextDocumentParams) -> Result<Vec<Url>> {
    let id: RequestId = format!(
        "{}#{}",
        params.text_document.uri, params.text_document.version
    )
    .into();

    Trace::receive(id.clone(), DidOpenTextDocument::METHOD);
    Trace::enrich(
        &id,
        FileUpdate {
            content: &params.content_changes[0].text,
        },
    );

    let (_, invalid) = server.world.update_file(
        params.text_document.uri.clone(),
        params.content_changes[0].text.clone(),
    )?;

    Trace::reply(id);
    Ok(server.world.uris(invalid).cloned().collect())
}
