use maelstrom::{Message, Node, Payload};

struct UniqueIds {
    current_id: usize,
    node_id: Option<String>,
}

impl UniqueIds {
    pub fn new() -> Self {
        Self {
            current_id: 1,
            node_id: None,
        }
    }
}

impl Node for UniqueIds {
    fn current_id(&mut self) -> &mut usize {
        &mut self.current_id
    }

    fn node_id_mut(&mut self) -> &mut Option<String> {
        &mut self.node_id
    }

    fn handle(&mut self, message: Message) -> anyhow::Result<Vec<Payload>> {
        if let Payload::Generate = message.body.payload {
            let current_id = self.current_id;
            let id = format!("{}-{}", self.node_id()?, current_id);

            Ok(vec![Payload::GenerateOk { id }])
        } else {
            Ok(Vec::new())
        }
    }
}

fn main() -> anyhow::Result<()> {
    UniqueIds::new().run()
}
