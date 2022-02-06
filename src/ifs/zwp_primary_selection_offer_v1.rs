use std::mem;
use crate::client::{Client, ClientError};
use crate::ifs::wl_seat::WlSeatGlobal;
use crate::ifs::zwp_primary_selection_source_v1::ZwpPrimarySelectionSourceV1;
use crate::object::Object;
use crate::utils::buffd::{MsgParser, MsgParserError};
use crate::utils::clonecell::CloneCell;
use std::ops::Deref;
use std::rc::Rc;
use thiserror::Error;
use crate::wire::zwp_primary_selection_offer_v1::*;
use crate::wire::ZwpPrimarySelectionOfferV1Id;

pub struct ZwpPrimarySelectionOfferV1 {
    pub id: ZwpPrimarySelectionOfferV1Id,
    pub client: Rc<Client>,
    pub source: CloneCell<Option<Rc<ZwpPrimarySelectionSourceV1>>>,
}

impl ZwpPrimarySelectionOfferV1 {
    pub fn create(
        client: &Rc<Client>,
        src: &Rc<ZwpPrimarySelectionSourceV1>,
        seat: &Rc<WlSeatGlobal>,
    ) -> Option<Rc<Self>> {
        let id = match client.new_id() {
            Ok(id) => id,
            Err(e) => {
                client.error(e);
                return None;
            }
        };
        let slf = Rc::new(Self {
            id,
            client: client.clone(),
            source: CloneCell::new(Some(src.clone())),
        });
        let mt = src.mime_types.borrow_mut();
        let mut sent_offer = false;
        seat.for_each_primary_selection_device(0, client.id, |device| {
            if !mem::replace(&mut sent_offer, true) {
                device.send_data_offer(slf.id);
            }
            for mt in mt.deref() {
                slf.send_offer(mt);
            }
            device.send_selection(id);
        });
        client.add_server_obj(&slf);
        if !sent_offer {
            let _ = client.remove_obj(&*slf);
            None
        } else {
            Some(slf)
        }
    }

    pub fn send_offer(self: &Rc<Self>, mime_type: &str) {
        self.client.event(Offer {
            self_id: self.id,
            mime_type,
        })
    }

    fn receive(&self, parser: MsgParser<'_, '_>) -> Result<(), ReceiveError> {
        let req: Receive = self.client.parse(self, parser)?;
        if let Some(src) = self.source.get() {
            src.send_send(req.mime_type, req.fd);
            src.client.flush();
        }
        Ok(())
    }

    fn disconnect(&self) {
        if let Some(src) = self.source.set(None) {
            src.clear_offer();
        }
    }

    fn destroy(&self, parser: MsgParser<'_, '_>) -> Result<(), DestroyError> {
        let _req: Destroy = self.client.parse(self, parser)?;
        self.disconnect();
        self.client.remove_obj(self)?;
        Ok(())
    }
}

object_base! {
    ZwpPrimarySelectionOfferV1, ZwpPrimarySelectionOfferV1Error;

    RECEIVE => receive,
    DESTROY => destroy,
}

impl Object for ZwpPrimarySelectionOfferV1 {
    fn num_requests(&self) -> u32 {
        DESTROY + 1
    }

    fn break_loops(&self) {
        self.disconnect();
    }
}

simple_add_obj!(ZwpPrimarySelectionOfferV1);

#[derive(Debug, Error)]
pub enum ZwpPrimarySelectionOfferV1Error {
    #[error(transparent)]
    ClientError(Box<ClientError>),
    #[error("Could not process `receive` request")]
    ReceiveError(#[from] ReceiveError),
    #[error("Could not process `destroy` request")]
    DestroyError(#[from] DestroyError),
}
efrom!(ZwpPrimarySelectionOfferV1Error, ClientError);

#[derive(Debug, Error)]
pub enum ReceiveError {
    #[error("Parsing failed")]
    ParseFailed(#[source] Box<MsgParserError>),
    #[error(transparent)]
    ClientError(Box<ClientError>),
}
efrom!(ReceiveError, ParseFailed, MsgParserError);
efrom!(ReceiveError, ClientError);

#[derive(Debug, Error)]
pub enum DestroyError {
    #[error("Parsing failed")]
    ParseFailed(#[source] Box<MsgParserError>),
    #[error(transparent)]
    ClientError(Box<ClientError>),
}
efrom!(DestroyError, ParseFailed, MsgParserError);
efrom!(DestroyError, ClientError);
