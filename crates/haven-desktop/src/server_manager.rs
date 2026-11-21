use std::process::Child;
use std::sync::Mutex;

pub struct LocalServerManager {
    child_process: Mutex<Option<Child>>,
}

impl LocalServerManager {
    pub fn new() -> Self {
        Self {
            child_process: Mutex::new(None),
        }
    }

    pub fn is_running(&self) -> bool {
        let mut guard = self.child_process.lock().unwrap();
        if let Some(ref mut child) = *guard {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    *guard = None;
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        let mut guard = self.child_process.lock().unwrap();
        if let Some(mut child) = guard.take() {
            child.kill()?;
            tracing::info!("Embedded Haven server process stopped");
        }
        Ok(())
    }
}

impl Default for LocalServerManager {
    fn default() -> Self {
        Self::new()
    }
}
