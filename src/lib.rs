use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::{
    io::{StdoutLock, Write},
    marker::PhantomData,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub struct Node {
    message_id: usize,
    init: Option<Init>,
}

pub struct MessageContext<'a, 'b, 'c, Payload> {
    node: &'a mut Node,
    stdout: &'b mut StdoutLock<'c>,
    src: String,
    in_reply_to: Option<usize>,
    payload: PhantomData<Payload>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            message_id: 1,
            init: None,
        }
    }
}

impl<'a, 'b, 'c, Payload> MessageContext<'a, 'b, 'c, Payload>
where
    Payload: Serialize,
{
    fn new(node: &'a mut Node, stdout: &'b mut StdoutLock<'c>, message: &Message<Payload>) -> Self {
        Self {
            node,
            stdout,
            src: message.src.clone(),
            in_reply_to: message.body.id.clone(),
            payload: PhantomData,
        }
    }

    pub fn reply(&mut self, payload: Payload) -> anyhow::Result<()> {
        let reply = Message {
            src: self.init()?.node_id.clone(),
            dst: self.src.clone(),
            body: Body {
                id: Some(self.next_id()),
                in_reply_to: self.in_reply_to,
                payload,
            },
        };

        self.send_message(reply)
    }

    pub fn send(&mut self, dst: String, payload: Payload) -> anyhow::Result<()> {
        let message = Message {
            src: self.init()?.node_id.clone(),
            dst,
            body: Body {
                id: Some(self.next_id()),
                in_reply_to: None,
                payload,
            },
        };

        self.send_message(message)
    }

    fn send_message(&mut self, message: Message<Payload>) -> anyhow::Result<()> {
        serde_json::to_writer(&mut *self.stdout, &message)
            .context("Failed to serialize output message to stdout")?;
        writeln!(&mut *self.stdout, "").context("Failed to write newline to stdout")?;

        Ok(())
    }

    pub fn initialize(&mut self, init: Init) {
        self.node.init = Some(init);
    }

    pub fn message_id(&self) -> usize {
        self.node.message_id
    }

    pub fn init(&self) -> anyhow::Result<&Init> {
        self.node
            .init
            .as_ref()
            .ok_or(anyhow!("Node was not initialized"))
    }

    fn next_id(&mut self) -> usize {
        let id = self.node.message_id;
        self.node.message_id += 1;
        id
    }
}

pub trait Handle<Payload>
where
    Payload: Serialize + for<'a> Deserialize<'a>,
{
    fn handle<'a, 'b, 'c>(
        &mut self,
        message: Message<Payload>,
        context: MessageContext<'a, 'b, 'c, Payload>,
    ) -> anyhow::Result<()>;

    fn run(&mut self) -> anyhow::Result<()> {
        let stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();
        let mut node = Node::default();

        let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<Payload>>();

        for input in inputs {
            let message = input.context("Maelstrom input could not be deserialized")?;
            let context = MessageContext::new(&mut node, &mut stdout, &message);
            self.handle(message, context)
                .context("An error ocurred while handling a message")?;
        }

        Ok(())
    }
}

/* trait NodeInternal<Payload>: Node<Payload>
where
    Payload: Serialize + for<'a> Deserialize<'a>,
{
    fn handle_internal(
        &mut self,
        message: Message<Payload>,
        stdout: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        let dst = message.src.clone();
        let in_reply_to = message.body.id.clone();

        let payloads = if let Payload::Init { node_id, .. } = message.body.payload {
            *self.node_id_mut() = Some(node_id);
            vec![Payload::InitOk]
        } else {
            self.handle(message)
                .context("Error while handling a message")?
        };

        for payload in payloads {
            let message = Message {
                src: self.node_id()?.to_string(),
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

impl<T, Payload> NodeInternal<Payload> for T where T: Node<Payload> + ?Sized {} */
