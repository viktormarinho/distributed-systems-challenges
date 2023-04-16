use std::io::{StdoutLock, Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Payload {
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
pub struct Body {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    // "type": "something_ok" will use the enum variant SomethingOk -
    // the serde rename all will convert it from snake case.
    // The enum payload contains a serde tag "type" to identify the json key.
    // Since the property "payload" does not exist we just use this enum as a helper, we need to flatten it
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body,
}

pub struct NodeState {
    pub next_msg_id: usize,
    pub node_id: String,
    pub other_node_ids: Vec<String>,
}

pub trait Node {
    fn handle_message(&mut self, msg: Message, stdout: &mut StdoutLock) -> anyhow::Result<()>;
    fn init(state: NodeState) -> Self;
    fn run(&mut self) -> anyhow::Result<()> {
        let stdin = std::io::stdin().lock();
        let mut inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();
    
        let mut stdout = std::io::stdout().lock();
    
        while let Some(input) = inputs.next() {
            self.handle_message(input?, &mut stdout)?;
        }
        Ok(())
    }
}

pub fn try_start<T>() -> T
    where T: Node {
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let msg = inputs.next().unwrap().unwrap();

    let mut state = NodeState::new();

    let info_reply = match msg.body.payload {
        Payload::Init { node_id, node_ids } => {
            let response = Message {
                src: msg.dst,
                dst: msg.src,
                body: Body {
                    payload: Payload::InitOk,
                    in_reply_to: msg.body.msg_id,
                    msg_id: Some(state.next_msg_id),
                },
            };
            state.node_id = node_id;
            state.other_node_ids = node_ids;
            Some(response)
        }
        _ => None,
    };

    state.next_msg_id += 1;
    reply_maelstrom(&mut stdout, info_reply.unwrap()).unwrap();

    return T::init(state);
}

pub fn reply_maelstrom(stdout: &mut StdoutLock, reply: Message) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *stdout, &reply)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

impl NodeState {
    pub fn new() -> Self {
        NodeState {
            next_msg_id: 0,
            node_id: "".to_string(),
            other_node_ids: Vec::new(),
        }
    }
}