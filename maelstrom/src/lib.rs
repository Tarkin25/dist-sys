use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

pub trait Node {
    fn handle(&mut self, message: Message) -> anyhow::Result<Vec<Payload>>;

    fn node_id(&mut self) -> &mut Option<String>;

    fn current_id(&mut self) -> &mut usize;

    fn run(&mut self) -> anyhow::Result<()> {
        let stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();

        let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

        for input in inputs {
            let input = input.context("Maelstrom input could not be deserialized")?;
            self.handle_internal(input, &mut stdout)?;
        }

        Ok(())
    }
}

trait NodeInternal: Node {
    fn handle_internal(&mut self, message: Message, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        let dst = message.src.clone();
        let in_reply_to = message.body.id.clone();

        let payloads = if let Payload::Init { node_id, .. } = message.body.payload {
            *self.node_id() = Some(node_id);
            vec![Payload::InitOk]
        } else {
            self.handle(message)
                .context("Error while handling a message")?
        };

        for payload in payloads {
            let message = Message {
                src: self
                    .node_id()
                    .as_ref()
                    .map(|s| s.clone())
                    .ok_or(anyhow!("self.node_id is not initialized"))?,
                dst: dst.clone(),
                body: Body {
                    id: Some(self.next_id()),
                    in_reply_to,
                    payload,
                },
            };

            serde_json::to_writer(&mut *stdout, &message)
                .context("Failed to serialze output message to stdout")?;
            writeln!(&mut *stdout, "").context("Failed to write newline to stdout")?;
        }

        Ok(())
    }

    fn next_id(&mut self) -> usize {
        let id = *self.current_id();
        *self.current_id() += 1;
        id
    }
}

impl<T> NodeInternal for T where T: Node + ?Sized {}
