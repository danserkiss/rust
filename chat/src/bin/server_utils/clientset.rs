use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender, channel};

#[derive(Clone, Debug, Default)]
pub struct ClientSet {
    pub set: Arc<Mutex<HashMap<String, Sender<String>>>>,
}
impl ClientSet {
    pub async fn add_new_user(&mut self, addr_str: String) -> (Sender<String>, Receiver<String>) {
        let client_set_cl = self.clone();
        let (tx_admin, rx_admin) = channel(1024);

        let mut mutex_gurad = client_set_cl.set.lock().await;
        mutex_gurad.insert(addr_str.clone(), tx_admin.clone());
        return (tx_admin, rx_admin);
    }

    pub async fn remove_user(&mut self, addr_str: String) {
        let mut mutex_gurad = self.set.lock().await;
        mutex_gurad.remove(&addr_str);
    }
}
