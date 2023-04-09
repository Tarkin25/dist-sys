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
    /// key: message value
    /// <br/>
    /// value: set containing all neighboring nodes the message has already been broadcasted to
    messages: HashMap<usize, HashSet<String>>,
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
                let notified_neighbors = self.messages.entry(message_number).or_default();
                let topology = self
                    .topology
                    .as_ref()
                    .ok_or(anyhow!("topology not initialized"))?;
                let node_id = &context.init()?.node_id;
                let neighbors = &topology[node_id];

                notified_neighbors.insert(message.src.clone());

                for neighbor in neighbors {
                    if !notified_neighbors.contains(neighbor) {
                        context.send(
                            neighbor.clone(),
                            Payload::Broadcast {
                                message: message_number,
                            },
                        )?;
                        notified_neighbors.insert(neighbor.clone());
                    }
                }

                context.reply(Payload::BroadcastOk)
            }
            Payload::Read => context.reply(Payload::ReadOk {
                messages: self.messages.keys().copied().collect(),
            }),
            _ => Ok(()),
        }
    }
}

fn main() -> anyhow::Result<()> {
    Broadcast::default().run()
}
