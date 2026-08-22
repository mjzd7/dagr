use dagr_core::{DagrError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexSlot {
    Blue,
    Green,
}

pub struct BlueGreenIndexManager {
    active_slot: Mutex<IndexSlot>,
    blue_version: Mutex<String>,
    green_version: Mutex<String>,
}

impl BlueGreenIndexManager {
    pub fn new(initial_version: &str) -> Self {
        Self {
            active_slot: Mutex::new(IndexSlot::Blue),
            blue_version: Mutex::new(initial_version.to_string()),
            green_version: Mutex::new(String::new()),
        }
    }

    pub fn get_active_slot(&self) -> IndexSlot {
        self.active_slot.lock().unwrap().clone()
    }

    pub fn get_active_version(&self) -> String {
        let slot = self.active_slot.lock().unwrap();
        match *slot {
            IndexSlot::Blue => self.blue_version.lock().unwrap().clone(),
            IndexSlot::Green => self.green_version.lock().unwrap().clone(),
        }
    }

    pub fn prepare_shadow_index(&self, new_version: &str) -> IndexSlot {
        let active = self.active_slot.lock().unwrap();
        match *active {
            IndexSlot::Blue => {
                *self.green_version.lock().unwrap() = new_version.to_string();
                IndexSlot::Green
            }
            IndexSlot::Green => {
                *self.blue_version.lock().unwrap() = new_version.to_string();
                IndexSlot::Blue
            }
        }
    }

    /// Atomically swaps the active index pointer once shadow indexing passes parity validation
    pub fn atomic_cutover(&self, target_slot: IndexSlot) -> Result<()> {
        let mut active = self.active_slot.lock().unwrap();
        if *active == target_slot {
            return Err(DagrError::Internal(
                "Target slot is already the active index slot".into(),
            ));
        }
        *active = target_slot;
        Ok(())
    }
}
