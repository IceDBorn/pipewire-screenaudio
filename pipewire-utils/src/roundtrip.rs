use std::{cell::Cell, rc::Rc};

use libspa::utils::result::AsyncSeq;
use pipewire::{
    core::{CoreRc, PW_ID_CORE},
    main_loop::MainLoopRc,
};

#[derive(Clone)]
pub struct Scheduler {
    sync_seq: Rc<Cell<Option<AsyncSeq>>>,
    mainloop: MainLoopRc,
    core: CoreRc,
}

impl Scheduler {
    pub fn new(mainloop: MainLoopRc, core: CoreRc) -> Self {
        Self {
            mainloop,
            core,
            sync_seq: Default::default(),
        }
    }

    pub fn schedule_roundtrip(&self) {
        self.sync_seq.replace(Some(self.core.sync(0).unwrap()));
    }

    pub fn run_until_sync(&self) {
        let done = Rc::new(Cell::new(false));

        let _listener_core = self
            .core
            .add_listener_local()
            .done({
                let done = done.clone();
                let mainloop = self.mainloop.clone();
				let sync_seq = self.sync_seq.clone();
                move |id, seq| {
                    if id == PW_ID_CORE && Some(seq) == sync_seq.get() {
                        done.set(true);
                        mainloop.quit();
                    }
                }
            })
            .register();

        while !done.get() {
            self.mainloop.run();
        }
    }
}
