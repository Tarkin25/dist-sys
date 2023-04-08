use maelstrom::{Message, Node, Payload};

struct Echo {
    current_id: usize,
    node_id: Option<String>,
}

impl Echo {
    pub fn new() -> Self {
        Self {
            current_id: 1,
            node_id: None,
        }
    }
}

impl Node for Echo {
    fn handle(&mut self, message: Message) -> anyhow::Result<Vec<Payload>> {
        match message.body.payload {
            Payload::Echo { echo } => Ok(vec![Payload::EchoOk { echo }]),
            _ => Ok(Vec::new()),
        }
    }

    fn current_id(&mut self) -> &mut usize {
        &mut self.current_id
    }

    fn node_id_mut(&mut self) -> &mut Option<String> {
        &mut self.node_id
    }
}

fn main() -> anyhow::Result<()> {
    Echo::new().run()
}
