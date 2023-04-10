use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::{
    io::{StdoutLock, Write},
    marker::PhantomData,
    sync::mpsc::Sender,
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

    fn run_with_generator(
        &mut self,
        generator: impl Fn(Sender<Message<Payload>>) -> anyhow::Result<()> + Send + Sync + 'static,
    ) -> anyhow::Result<()>
    where
        Payload: Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut stdout = std::io::stdout().lock();
        let mut node = Node::default();

        let read_stdin_handle = {
            let tx = tx.clone();

            std::thread::spawn(move || {
                let stdin = std::io::stdin().lock();
                let inputs =
                    serde_json::Deserializer::from_reader(stdin).into_iter::<Message<Payload>>();

                for input in inputs {
                    let message = input.context("Maelstrom input could not be deserialized")?;
                    if let Err(_) = tx.send(message) {
                        return Ok::<_, anyhow::Error>(());
                    }
                }

                Ok(())
            })
        };

        let generator_handle = std::thread::spawn(move || generator(tx));

        for message in rx {
            let context = MessageContext::new(&mut node, &mut stdout, &message);
            self.handle(message, context)
                .context("An error ocurred while handling a message")?;
        }

        read_stdin_handle
            .join()
            .expect("read-input thread panicked")
            .context("read-input thread errorred")?;
        generator_handle
            .join()
            .expect("generator thread panicked")
            .context("generator thread errorred")?;

        Ok(())
    }
}
