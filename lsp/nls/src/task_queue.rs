use std::{
    collections::{HashMap, HashSet, VecDeque},
    str::FromStr,
};

use anyhow::Result;
use log::warn;
use lsp_server::{Message, Notification};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, Url,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
    },
    request::{ExecuteCommand, Request},
};

#[derive(Debug, Clone)]
pub enum Task {
    /// Handle an LSP request
    HandleRequest(lsp_server::Request),
    /// Handle a document synchronization notification.
    HandleDocumentSync(DocumentSync),
    /// Publish diagnostics for a file
    Diagnostics(DiagnosticsRequest),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Normal,
    High,
}

#[derive(Debug, Clone)]
pub struct DiagnosticsRequest {
    pub uri: Url,
    pub priority: Priority,
}

#[derive(Debug, Clone)]
pub enum DocumentSync {
    DidOpenTextDocument(DidOpenTextDocumentParams),
    DidCloseTextDocument(DidCloseTextDocumentParams),
    DidChangeTextDocument(DidChangeTextDocumentParams),
}

impl DocumentSync {
    /// Get the URI of the document that this task applies to.
    fn uri(&self) -> &Url {
        match self {
            DocumentSync::DidChangeTextDocument(params) => &params.text_document.uri,
            DocumentSync::DidOpenTextDocument(params) => &params.text_document.uri,
            DocumentSync::DidCloseTextDocument(params) => &params.text_document.uri,
        }
    }

    fn is_did_change_text_document(&self) -> bool {
        matches!(self, DocumentSync::DidChangeTextDocument(_))
    }
}

pub struct TaskQueue {
    /// LSP requests that need to be handled. These are processed in the order that they are
    /// received.
    requests: VecDeque<lsp_server::Request>,
    /// The set of files that are currently open in memory. This is used to prioritize tasks
    /// for open files.
    open_files: HashSet<Url>,
    /// Document synchronization tasks that need to be handled. These are created by LSP
    /// notifications like DidChangeTextDocument.
    sync_tasks: HashMap<Url, VecDeque<DocumentSync>>,
    /// The set of files that need updated diagnostics published for them.
    diagnostics: HashSet<Url>,
}

impl TaskQueue {
    pub fn new() -> Self {
        TaskQueue {
            requests: VecDeque::new(),
            open_files: HashSet::new(),
            sync_tasks: HashMap::new(),
            diagnostics: HashSet::new(),
        }
    }

    pub fn queue_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Request(req) => {
                self.requests.push_back(req);
                Ok(())
            }
            Message::Notification(notification) => self.queue_notification(notification),
            Message::Response(_) => Ok(()),
        }
    }

    fn add_sync_task(&mut self, task: DocumentSync) {
        if let Some(queue) = self.sync_tasks.get_mut(task.uri()) {
            queue.push_back(task);
        } else {
            let uri = task.uri().clone();
            let mut queue = VecDeque::new();
            queue.push_back(task);
            self.sync_tasks.insert(uri, queue);
        }
    }

    pub fn add_diagnostics_task(&mut self, uri: Url) {
        self.diagnostics.insert(uri);
    }

    fn queue_notification(&mut self, notification: Notification) -> Result<()> {
        match notification.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params =
                    serde_json::from_value::<DidOpenTextDocumentParams>(notification.params)?;
                self.open_files.insert(params.text_document.uri.clone());
                self.add_sync_task(DocumentSync::DidOpenTextDocument(params));
            }
            DidCloseTextDocument::METHOD => {
                let params =
                    serde_json::from_value::<DidCloseTextDocumentParams>(notification.params)?;
                self.open_files.remove(&params.text_document.uri);
                self.add_sync_task(DocumentSync::DidCloseTextDocument(params));
            }
            DidChangeTextDocument::METHOD => {
                let params =
                    serde_json::from_value::<DidChangeTextDocumentParams>(notification.params)?;
                self.add_sync_task(DocumentSync::DidChangeTextDocument(params));
            }
            method => warn!("No handler for notification type {}", method),
        };
        Ok(())
    }

    /// Get the next file that needs its diagnostics refreshed. This has a preference for files
    /// that are open in memory since those files are currently being edited and are of greater
    /// relevance. If no open files are out of date it will pick another out of date file in an
    /// arbitrary order.
    fn next_diagnostic(&mut self) -> Option<Task> {
        if let Some(uri) = self
            .open_files
            .iter()
            .find(|file| self.diagnostics.contains(file))
            .cloned()
        {
            self.diagnostics.remove(&uri);
            Some(Task::Diagnostics(DiagnosticsRequest {
                uri: uri,
                priority: Priority::High,
            }))
        } else if let Some(uri) = self.diagnostics.iter().next().cloned() {
            self.diagnostics.remove(&uri);
            Some(Task::Diagnostics(DiagnosticsRequest {
                uri: uri,
                priority: Priority::Normal,
            }))
        } else {
            None
        }
    }

    /// Get the next request to handle if there's one pending.
    fn next_request(&mut self) -> Option<Task> {
        enum RequestCategory {
            EvalCommand(Url),
            TextDocumentUri(Url),
            Other,
        }

        fn categorize(req: &lsp_server::Request) -> RequestCategory {
            let is_eval_command = req.method == ExecuteCommand::METHOD
                && req.params["command"].as_str() == Some("eval");
            if is_eval_command {
                let uri = req.params["arguments"]
                    .as_array()
                    .and_then(|args| args.first())
                    .and_then(|arg| arg["uri"].as_str())
                    .and_then(|url| Url::from_str(url).ok());
                return match uri {
                    Some(uri) => RequestCategory::EvalCommand(uri),
                    None => RequestCategory::Other,
                };
            };

            // This should get the document URI for any request type extending TextDocumentPositionParams
            // in the LSP spec. https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentPositionParams
            let uri = req.params["textDocument"]["uri"]
                .as_str()
                .and_then(|uri| Url::from_str(uri).ok());
            match uri {
                Some(uri) => RequestCategory::TextDocumentUri(uri),
                None => RequestCategory::Other,
            }
        }

        if let Some(next_request) = self.requests.front() {
            let task = match categorize(next_request) {
                RequestCategory::EvalCommand(uri) => {
                    // The eval command publishes the new diagnostics for a file
                    // synchronously, so if there were a pending update to the diagnostics for a
                    // file, we can remove it.
                    self.diagnostics.remove(&uri);
                    Task::HandleRequest(self.requests.pop_front().unwrap())
                }
                RequestCategory::TextDocumentUri(uri) => {
                    // Most of the request handlers expect the file to be parsed and typechecked
                    // already before the handler is run. If the request specifies a file URI, and
                    // the diagnostics on that URI are out of date, then the diagnostics task will
                    // need to be run first before the request can be handled. That will ensure
                    // that parsing and typechecking are current before the request handler is run.
                    if self.diagnostics.contains(&uri) {
                        self.diagnostics.remove(&uri);
                        Task::Diagnostics(DiagnosticsRequest {
                            uri: uri,
                            priority: Priority::High,
                        })
                    } else {
                        Task::HandleRequest(self.requests.pop_front().unwrap())
                    }
                }
                RequestCategory::Other => Task::HandleRequest(self.requests.pop_front().unwrap()),
            };
            Some(task)
        } else {
            None
        }
    }

    /// Gets the document synchronization task that needs to be done if there is one.
    fn next_sync_task(&mut self) -> Option<Task> {
        // All document synchronization tasks get completed before doing anything else anyway so
        // these are just done in an arbitrary order.
        let next_url = self.sync_tasks.keys().next().cloned();
        next_url.map(|url| {
            // unwrap(): We know next_url was in the keys so the entry must exist.
            let queue = self.sync_tasks.get_mut(&url).unwrap();
            // unwrap(): If an entry is created for a url, it will always have at least one
            // task, and if the queue for that url is emptied out, the entry will be deleted. This
            // should never be empty.
            let mut next_task = queue.pop_front().unwrap();
            if next_task.is_did_change_text_document() {
                while queue
                    .front()
                    .map(|it| it.is_did_change_text_document())
                    .unwrap_or(false)
                {
                    // unwrap(): This only runs when we've found an entry with `front()`
                    next_task = queue.pop_front().unwrap();
                }
            };

            // The entry removed because everything previous assumes that an entry must have at least
            // one task.
            if queue.is_empty() {
                self.sync_tasks.remove(&url);
            }

            Task::HandleDocumentSync(next_task)
        })
    }

    pub fn next_task(&mut self) -> Option<Task> {
        self.next_sync_task()
            .or_else(|| self.next_request())
            .or_else(|| self.next_diagnostic())
    }
}
