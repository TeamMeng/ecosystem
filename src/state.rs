use crate::model::Task;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};

pub struct AppState {
    pub tasks: Arc<Mutex<HashMap<u64, Task>>>,
    pub task_tx: mpsc::Sender<u64>,
    pub next_id: Arc<Mutex<u64>>,
}
