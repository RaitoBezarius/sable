use sable_network::prelude::*;

use crate::{
    command::Command,
    messages::{Numeric, message},
};

pub trait CommandExt
{
    fn notice(&self, text: impl ToString);
    fn numeric(&self, numeric: impl Numeric);
    fn new_event(&self, target: impl Into<ObjectId>, detail: impl Into<EventDetails>);
}

impl<T: Command + ?Sized> CommandExt for T
{
    fn notice(&self, text: impl ToString)
    {
        let n = message::Notice::new(self.response_source(), &self.source(), &text.to_string());
        self.response(&n);
    }

    fn numeric(&self, numeric: impl Numeric)
    {
        self.response(&numeric.format_for(self.response_source(), &self.source()));
    }

    fn new_event(&self, target: impl Into<ObjectId>, detail: impl Into<EventDetails>)
    {
        self.server().node().submit_event(target, detail);
    }
}