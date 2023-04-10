use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

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
    InitiateGossip,
    Gossip { messages: HashSet<usize> },
    GossipOk { messages: HashSet<usize> },
}

#[derive(Default)]
struct Broadcast {
    neighbors: Option<Vec<String>>,
    messages: HashSet<usize>,
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
            Payload::Topology { mut topology } => {
                self.neighbors = topology.remove(&context.init()?.node_id);
                context.reply(Payload::TopologyOk)
            }
            Payload::Broadcast { message } => {
                self.messages.insert(message);
                context.reply(Payload::BroadcastOk)
            }
            Payload::Read => context.reply(Payload::ReadOk {
                messages: self.messages.iter().copied().collect(),
            }),
            Payload::InitiateGossip => {
                if let Some(neighbors) = self.neighbors.as_ref() {
                    for neighbor in neighbors {
                        context.send(
                            neighbor.clone(),
                            Payload::Gossip {
                                messages: self.messages.clone(),
                            },
                        )?;
                    }
                }

                Ok(())
            }
            Payload::Gossip { messages } => {
                self.messages.extend(messages);
                context.reply(Payload::GossipOk {
                    messages: self.messages.clone(),
                })
            }
            Payload::GossipOk { messages } => {
                self.messages.extend(messages);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

fn main() -> anyhow::Result<()> {
    Broadcast::default().run_with_generator(|sender| loop {
        let message = Message {
            dst: "internal".into(),
            src: "internal".into(),
            body: Body {
                id: None,
                in_reply_to: None,
                payload: Payload::InitiateGossip,
            },
        };

        if let Err(_) = sender.send(message) {
            return Ok(());
        }

        std::thread::sleep(Duration::from_millis(500));
    })
}
