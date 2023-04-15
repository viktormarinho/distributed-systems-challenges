use std::io::{StdoutLock, Write};

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum Payload {
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Body {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
    // "type": "something_ok" will use the enum variant SomethingOk -
    // the serde rename all will convert it from snake case.
    // The enum payload contains a serde tag "type" to identify the json key.
    // Since the property "payload" does not exist we just use this enum as a helper, we need to flatten it
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

enum NodeStatus {
    Active,
    Inactive,
}

struct EchoNode {
    next_msg_id: usize,
    node_id: String,
    other_node_ids: Vec<String>,
    status: NodeStatus,
}

impl EchoNode {
    pub fn new() -> Self {
        Self {
            next_msg_id: 0,
            node_id: "echo".to_string(),
            other_node_ids: Vec::new(),
            status: NodeStatus::Inactive,
        }
    }

    pub fn try_start(&mut self, message: Message) -> Option<Message> {
        match message.body.payload {
            Payload::Init { node_id, node_ids } => {
                let response = Message {
                    src: message.dst,
                    dst: message.src,
                    body: Body {
                        payload: Payload::InitOk,
                        in_reply_to: message.body.msg_id,
                        msg_id: Some(self.next_msg_id),
                    },
                };
                self.node_id = node_id;
                self.other_node_ids = node_ids;
                self.status = NodeStatus::Active;
                Some(response)
            }
            _ => None,
        }
    }

    pub fn process(&mut self, message: Message, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        match self.status {
            NodeStatus::Inactive => {
                let reply = self.try_start(message);
                if let Some(reply) = reply {
                    serde_json::to_writer(&mut *stdout, &reply)?;
                    stdout.write_all(b"\n")?;
                }
            }
            NodeStatus::Active => {
                match message.body.payload {
                    Payload::Echo { echo } => {
                        let reply = Message {
                            dst: message.src,
                            src: message.dst,
                            body: Body {
                                msg_id: Some(self.next_msg_id),
                                in_reply_to: message.body.msg_id,
                                payload: Payload::EchoOk { echo },
                            },
                        };
                        serde_json::to_writer(&mut *stdout, &reply)?;
                        stdout.write_all(b"\n")?;
                    }
                    Payload::Init { .. } => bail!("Node already active"),
                    Payload::InitOk => bail!("InitOk should not be processed"),
                    Payload::EchoOk { .. } => {}
                };
            }
        }
        self.next_msg_id += 1;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut echo_node = EchoNode::new();

    let mut stdout = std::io::stdout().lock();

    while let Some(input) = inputs.next() {
        echo_node.process(input?, &mut stdout)?;
    }
    Ok(())
}
