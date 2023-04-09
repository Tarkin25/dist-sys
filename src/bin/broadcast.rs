use std::collections::{HashMap, HashSet};

use anyhow::anyhow;
use dist_sys::*;
use serde::{Deserialize, Serialize};

type Topology = HashMap<String, Vec<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init(Init),
    InitOk,
    Broadcast { message: usize },
    BroadcastOk,
    Read,
    ReadOk { messages: Vec<usize> },
    Topology { topology: Topology },
    TopologyOk,
}

#[derive(Default)]
struct Broadcast {
    topology: Option<Topology>,
    read_messages: HashSet<usize>,
}

impl Broadcast {
    fn topology(&self) -> anyhow::Result<&Topology> {
        self.topology
            .as_ref()
            .ok_or(anyhow!("Topology is not initialized"))
    }
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
            Payload::Broadcast {
                message: message_number,
            } => {
                self.read_messages.insert(message_number);
                let topology = self.topology()?;
                let node_id = &context.init()?.node_id;
                let neighbors = &topology[node_id];

                for neighbor in neighbors
                    .into_iter()
                    .filter(|neighbor| *neighbor != &message.src)
                {
                    context.send(
                        neighbor.clone(),
                        Payload::Broadcast {
                            message: message_number,
                        },
                    )?;
                }

                context.reply(Payload::BroadcastOk)
            }
            Payload::Read => context.reply(Payload::ReadOk {
                messages: self.read_messages.clone().into_iter().collect(),
            }),
            _ => Ok(()),
        }
    }
}

fn main() -> anyhow::Result<()> {
    Broadcast::default().run()
}
