use crate::client::{Client, ClientError};
use crate::globals::{Global, GlobalName};
use crate::ifs::zxdg_output_v1::ZxdgOutputV1;
use crate::leaks::Tracker;
use crate::object::Object;
use crate::utils::buffd::MsgParser;
use crate::utils::buffd::MsgParserError;
use crate::wire::zxdg_output_manager_v1::*;
use crate::wire::ZxdgOutputManagerV1Id;
use std::rc::Rc;
use thiserror::Error;

pub struct ZxdgOutputManagerV1Global {
    name: GlobalName,
}

pub struct ZxdgOutputManagerV1 {
    pub id: ZxdgOutputManagerV1Id,
    pub client: Rc<Client>,
    pub version: u32,
    pub tracker: Tracker<Self>,
}

impl ZxdgOutputManagerV1Global {
    pub fn new(name: GlobalName) -> Self {
        Self { name }
    }

    fn bind_(
        self: Rc<Self>,
        id: ZxdgOutputManagerV1Id,
        client: &Rc<Client>,
        version: u32,
    ) -> Result<(), ZxdgOutputManagerV1Error> {
        let obj = Rc::new(ZxdgOutputManagerV1 {
            id,
            client: client.clone(),
            version,
            tracker: Default::default(),
        });
        track!(client, obj);
        client.add_client_obj(&obj)?;
        Ok(())
    }
}

impl ZxdgOutputManagerV1 {
    fn destroy(&self, parser: MsgParser<'_, '_>) -> Result<(), DestroyError> {
        let _req: Destroy = self.client.parse(self, parser)?;
        self.client.remove_obj(self)?;
        Ok(())
    }

    fn get_xdg_output(self: &Rc<Self>, parser: MsgParser<'_, '_>) -> Result<(), GetXdgOutputError> {
        let req: GetXdgOutput = self.client.parse(&**self, parser)?;
        let output = self.client.lookup(req.output)?;
        let xdg_output = Rc::new(ZxdgOutputV1 {
            id: req.id,
            version: self.version,
            client: self.client.clone(),
            output: output.clone(),
            tracker: Default::default(),
        });
        track!(self.client, xdg_output);
        self.client.add_client_obj(&xdg_output)?;
        xdg_output.send_updates();
        output.xdg_outputs.set(req.id, xdg_output);
        Ok(())
    }
}

global_base!(
    ZxdgOutputManagerV1Global,
    ZxdgOutputManagerV1,
    ZxdgOutputManagerV1Error
);

impl Global for ZxdgOutputManagerV1Global {
    fn singleton(&self) -> bool {
        true
    }

    fn version(&self) -> u32 {
        3
    }
}

simple_add_global!(ZxdgOutputManagerV1Global);

object_base! {
    ZxdgOutputManagerV1, ZxdgOutputManagerV1Error;

    DESTROY => destroy,
    GET_XDG_OUTPUT => get_xdg_output,
}

simple_add_obj!(ZxdgOutputManagerV1);

impl Object for ZxdgOutputManagerV1 {
    fn num_requests(&self) -> u32 {
        GET_XDG_OUTPUT + 1
    }
}

#[derive(Debug, Error)]
pub enum ZxdgOutputManagerV1Error {
    #[error(transparent)]
    ClientError(Box<ClientError>),
    #[error("Could not process a `destroy` request")]
    DestroyError(#[from] DestroyError),
    #[error("Could not process a `get_xdg_output` request")]
    GetXdgOutputError(#[from] GetXdgOutputError),
}
efrom!(ZxdgOutputManagerV1Error, ClientError);

#[derive(Debug, Error)]
pub enum DestroyError {
    #[error("Parsing failed")]
    ParseError(#[source] Box<MsgParserError>),
    #[error(transparent)]
    ClientError(Box<ClientError>),
}
efrom!(DestroyError, ParseError, MsgParserError);
efrom!(DestroyError, ClientError);

#[derive(Debug, Error)]
pub enum GetXdgOutputError {
    #[error("Parsing failed")]
    ParseError(#[source] Box<MsgParserError>),
    #[error(transparent)]
    ClientError(Box<ClientError>),
}
efrom!(GetXdgOutputError, ParseError, MsgParserError);
efrom!(GetXdgOutputError, ClientError);