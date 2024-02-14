use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CtrlCHandler {
    was_pressed: Arc<AtomicBool>,
}

impl CtrlCHandler {
    pub fn new() -> Self {
        let this = Self {
            was_pressed: Arc::new(AtomicBool::new(false)),
        };
        let res = ctrlc::set_handler({
            let this = this.clone();
            move || {
                this.was_pressed.store(true, Ordering::SeqCst);
            }
        });
        match res {
            Ok(()) | Err(ctrlc::Error::NoSuchSignal(_)) | Err(ctrlc::Error::System(_)) => {}
            Err(ctrlc::Error::MultipleHandlers) => panic!("Ctrl+c handler was set multiple times"),
        }
        this
    }

    pub fn mock() -> Self {
        Self {
            was_pressed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn catch(&self) -> Result<(), CtrlCError> {
        if self.was_pressed.swap(false, Ordering::SeqCst) {
            Err(CtrlCError)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("ctr+c was pressed")]
pub struct CtrlCError;
