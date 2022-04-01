use crate::client::{Client, ClientError};
use crate::globals::{Global, GlobalName};
use crate::ifs::jay_log_file::JayLogFile;
use crate::leaks::Tracker;
use crate::object::Object;
use crate::utils::buffd::{MsgParser, MsgParserError};
use crate::wire::jay_compositor::*;
use crate::wire::JayCompositorId;
use std::rc::Rc;
use log::Level;
use thiserror::Error;
use crate::cli::CliLogLevel;

pub struct JayCompositorGlobal {
    name: GlobalName,
}

impl JayCompositorGlobal {
    pub fn new(name: GlobalName) -> Self {
        Self { name }
    }

    fn bind_(
        self: Rc<Self>,
        id: JayCompositorId,
        client: &Rc<Client>,
        _version: u32,
    ) -> Result<(), JayCompositorError> {
        let obj = Rc::new(JayCompositor {
            id,
            client: client.clone(),
            tracker: Default::default(),
        });
        track!(client, obj);
        client.add_client_obj(&obj)?;
        Ok(())
    }
}

global_base!(JayCompositorGlobal, JayCompositor, JayCompositorError);

impl Global for JayCompositorGlobal {
    fn singleton(&self) -> bool {
        true
    }

    fn version(&self) -> u32 {
        1
    }

    fn secure(&self) -> bool {
        true
    }
}

simple_add_global!(JayCompositorGlobal);

pub struct JayCompositor {
    id: JayCompositorId,
    client: Rc<Client>,
    tracker: Tracker<Self>,
}

impl JayCompositor {
    fn destroy(&self, parser: MsgParser<'_, '_>) -> Result<(), DestroyError> {
        let _req: Destroy = self.client.parse(self, parser)?;
        self.client.remove_obj(self)?;
        Ok(())
    }

    fn get_log_file(&self, parser: MsgParser<'_, '_>) -> Result<(), GetLogFileError> {
        let req: GetLogFile = self.client.parse(self, parser)?;
        let log_file = Rc::new(JayLogFile::new(req.id, &self.client));
        track!(self.client, log_file);
        self.client.add_client_obj(&log_file)?;
        log_file.send_path(self.client.state.logger.path());
        Ok(())
    }

    fn quit(&self, parser: MsgParser<'_, '_>) -> Result<(), QuitError> {
        let _req: Quit = self.client.parse(self, parser)?;
        log::info!("Quitting");
        self.client.state.el.stop();
        Ok(())
    }

    fn set_log_level(&self, parser: MsgParser<'_, '_>) -> Result<(), SetLogLevelError> {
        let req: SetLogLevel = self.client.parse(self, parser)?;
        const ERROR: u32 = CliLogLevel::Error as u32;
        const WARN: u32 = CliLogLevel::Warn as u32;
        const INFO: u32 = CliLogLevel::Info as u32;
        const DEBUG: u32 = CliLogLevel::Debug as u32;
        const TRACE: u32 = CliLogLevel::Trace as u32;
        let level = match req.level {
            ERROR => Level::Error,
            WARN => Level::Warn,
            INFO => Level::Info,
            DEBUG => Level::Debug,
            TRACE => Level::Trace,
            _ => return Err(SetLogLevelError::UnknownLogLevel(req.level)),
        };
        self.client.state.logger.set_level(level);
        Ok(())
    }
}

object_base! {
    JayCompositor, JayCompositorError;

    DESTROY => destroy,
    GET_LOG_FILE => get_log_file,
    QUIT => quit,
    SET_LOG_LEVEL => set_log_level,
}

impl Object for JayCompositor {
    fn num_requests(&self) -> u32 {
        SET_LOG_LEVEL + 1
    }
}

simple_add_obj!(JayCompositor);

#[derive(Debug, Error)]
pub enum JayCompositorError {
    #[error("Could not process a `destroy` request")]
    DestroyError(#[from] DestroyError),
    #[error("Could not process a `get_log_file` request")]
    GetLogFileError(#[from] GetLogFileError),
    #[error("Could not process a `quit` request")]
    QuitError(#[from] QuitError),
    #[error("Could not process a `set_log_level` request")]
    SetLogLevelError(#[from] SetLogLevelError),
    #[error(transparent)]
    ClientError(Box<ClientError>),
}
efrom!(JayCompositorError, ClientError);

#[derive(Debug, Error)]
pub enum DestroyError {
    #[error("Parsing failed")]
    MsgParserError(#[source] Box<MsgParserError>),
    #[error(transparent)]
    ClientError(Box<ClientError>),
}
efrom!(DestroyError, ClientError);
efrom!(DestroyError, MsgParserError);

#[derive(Debug, Error)]
pub enum GetLogFileError {
    #[error("Parsing failed")]
    MsgParserError(#[source] Box<MsgParserError>),
    #[error(transparent)]
    ClientError(Box<ClientError>),
}
efrom!(GetLogFileError, ClientError);
efrom!(GetLogFileError, MsgParserError);

#[derive(Debug, Error)]
pub enum QuitError {
    #[error("Parsing failed")]
    MsgParserError(#[source] Box<MsgParserError>),
}
efrom!(QuitError, MsgParserError);

#[derive(Debug, Error)]
pub enum SetLogLevelError {
    #[error("Parsing failed")]
    MsgParserError(#[source] Box<MsgParserError>),
    #[error("Unknown log level {0}")]
    UnknownLogLevel(u32),
}
efrom!(SetLogLevelError, MsgParserError);