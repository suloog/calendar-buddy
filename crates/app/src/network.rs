//! Whether the app is online, as the system says (`MOD-02-F14`), for the UI thread to read.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use calendar_core::platform::Network;
use eframe::egui;

pub struct Online(Arc<AtomicBool>);

impl Online {
    /// Follows the system's word; until it comes, the app counts as online.
    pub fn watch(ctx: &egui::Context, network: &dyn Network) -> Self {
        let online = Arc::new(AtomicBool::new(true));
        let (flag, ctx) = (online.clone(), ctx.clone());
        network.watch(Box::new(move |now| {
            if flag.swap(now, Ordering::Relaxed) != now {
                ctx.request_repaint();
            }
        }));
        Online(online)
    }

    pub fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}
