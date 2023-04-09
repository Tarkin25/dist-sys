use dist_sys::*;
use serde::{Deserialize, Serialize};

struct Echo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init(Init),
    InitOk,
    Echo { echo: String },
    EchoOk { echo: String },
}

impl Handle<Payload> for Echo {
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
            Payload::Echo { echo } => context.reply(Payload::EchoOk { echo }),
            _ => Ok(()),
        }
    }
}

fn main() -> anyhow::Result<()> {
    Echo.run()
}
