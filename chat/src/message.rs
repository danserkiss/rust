use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Message {
    KeepAlive,
    UserJoined(UserAddr),
    UserLeft(UserAddr),
    ChatMessage(ChatMessage),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub addr: String,
    pub msg: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserAddr {
    pub addr: String,
}
impl UserAddr {
    pub fn new(m_addr: String) -> Self {
        UserAddr { addr: m_addr }
    }
}

impl ChatMessage {
    pub fn new(m_addr: String, m_msg: String) -> Self {
        ChatMessage {
            addr: m_addr,
            msg: m_msg,
        }
    }
}

impl Message {
    pub fn keep_alive() -> Message {
        Message::KeepAlive
    }
    pub fn user_joined(addr: String) -> Message {
        Message::UserJoined(UserAddr { addr })
    }
    pub fn user_left(addr: String) -> Message {
        Message::UserLeft(UserAddr { addr })
    }
    pub fn chat_message(addr: String, msg: String) -> Message {
        Message::ChatMessage(ChatMessage { addr, msg })
    }

    pub fn to_json(self) -> Result<std::string::String, serde_json::Error> {
        serde_json::to_string(&self)
    }
    pub fn from_json(json: String) -> Result<Message, serde_json::Error> {
        serde_json::from_str::<Message>(&json)
    }
}
