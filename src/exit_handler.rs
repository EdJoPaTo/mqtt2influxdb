use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct ExitHandler(Arc<AtomicBool>);

impl ExitHandler {
    pub fn new() -> Self {
        let quit = Arc::new(AtomicBool::new(false));
        ctrlc::set_handler({
            let quit = quit.clone();
            move || {
                if quit.swap(true, Ordering::SeqCst) {
                    eprintln!("Force exit");
                    std::process::exit(-1);
                }
                eprintln!("Exiting...");
            }
        })
        .expect("Failed to set Ctrl-C handler");
        Self(quit)
    }

    pub fn is_exiting(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
