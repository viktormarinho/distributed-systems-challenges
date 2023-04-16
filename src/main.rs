use std::io::StdoutLock;

use anyhow::bail;
use distributed_systems_chall::{Body, Message, Node, NodeState, Payload, reply_maelstrom, try_start};

struct EchoNode {
    state: NodeState,
}

impl Node for EchoNode {
    fn init(state: NodeState) -> Self {
        Self { state }
    }
    fn handle_message(&mut self, message: Message, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        match message.body.payload {
            Payload::Echo { echo } => {
                let reply = Message {
                    dst: message.src,
                    src: message.dst,
                    body: Body {
                        msg_id: Some(self.state.next_msg_id),
                        in_reply_to: message.body.msg_id,
                        payload: Payload::EchoOk { echo },
                    },
                };
                reply_maelstrom(stdout, reply)?;
            }
            Payload::Init { .. } => bail!("Node already active"),
            Payload::InitOk => bail!("InitOk should not be processed"),
            Payload::EchoOk { .. } => {},
            _ => bail!("Echo Node does not support this type of payload: {:?}", message.body.payload),
        }
        self.state.next_msg_id += 1;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut node = try_start::<EchoNode>();

    node.run()?;
    Ok(())
}
