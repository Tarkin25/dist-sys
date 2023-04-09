use dist_sys::*;
use serde::{Deserialize, Serialize};

struct UniqueIds;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init(Init),
    InitOk,
    Generate,
    GenerateOk { id: String },
}

impl Handle<Payload> for UniqueIds {
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
            Payload::Generate => {
                let id = format!("{}-{}", &context.init()?.node_id, context.message_id());
                context.reply(Payload::GenerateOk { id })
            }
            _ => Ok(()),
        }
    }
}

fn main() -> anyhow::Result<()> {
    UniqueIds.run()
}
