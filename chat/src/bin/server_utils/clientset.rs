use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Debug, Default)]
pub struct ClientSet {
    pub set: Arc<Mutex<HashMap<String, Sender<String>>>>,
}
