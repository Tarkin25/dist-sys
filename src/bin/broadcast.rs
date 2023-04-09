use std::collections::HashMap;

use dist_sys::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init(Init),
    InitOk,
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

#[derive(Default)]
struct Broadcast {
    topology: Option<HashMap<String, Vec<String>>>,
    read_messages: Vec<usize>,
}

impl Handle<Payload> for Broadcast {
    fn handle<'a, 'b, 'c>(
        &mut self,
        message: Message<Payload>,
        mut context: MessageContext<'a, 'b, 'c, Payload>,
    ) -> anyhow::Result<()> {
        match message.body.payload {
            Payload::Init(init) => {
                context.initialize(init);
                context.reply(Payload::InitOk)
            }
            Payload::Topology { topology } => {
                self.topology = Some(topology);
                context.reply(Payload::TopologyOk)
            }
            Payload::Broadcast { message } => {
                self.read_messages.push(message);
                context.reply(Payload::BroadcastOk)
            }
            Payload::Read => context.reply(Payload::ReadOk {
                messages: self.read_messages.clone(),
            }),
            _ => Ok(()),
        }
    }
}

fn main() -> anyhow::Result<()> {
    Broadcast::default().run()
}
