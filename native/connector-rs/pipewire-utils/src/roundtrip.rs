use std::{cell::Cell, fmt::Debug, rc::Rc};

use libspa::utils::result::AsyncSeq;
use pipewire::{
    core::{CoreRc, PW_ID_CORE},
    main_loop::MainLoopRc,
};
use tracing::instrument;

use crate::cancellation_signal::CancellationSignal;

#[derive(Clone)]
pub struct Scheduler {
    sync_seq: Rc<Cell<Option<AsyncSeq>>>,
    mainloop: MainLoopRc,
    core: CoreRc,
    done: Rc<Cell<bool>>,
}

pub struct StopSettings {
    on_last_roundtrip: bool,
    signal_receiver: Option<CancellationSignal>,
}

impl Debug for StopSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "StopSettings {{ on_last_roundtrip: {}, signal_receiver: {} }}",
            self.on_last_roundtrip,
            self.signal_receiver
                .as_ref()
                .map(|_| "Some(...)")
                .unwrap_or("None")
        ))
    }
}

impl From<CancellationSignal> for StopSettings {
    fn from(value: CancellationSignal) -> Self {
        StopSettingsBuilder::default().stop_on_signal(value).build()
    }
}

pub struct StopSettingsBuilder {
    on_last_roundtrip: bool,
    signal_receiver: Option<CancellationSignal>,
}

#[allow(clippy::derivable_impls)]
impl Default for StopSettingsBuilder {
    fn default() -> Self {
        Self {
            on_last_roundtrip: false,
            signal_receiver: None,
        }
    }
}
impl StopSettingsBuilder {
    pub fn stop_on_last_roundtrip(mut self) -> Self {
        self.on_last_roundtrip = true;
        self
    }

    pub fn set_stop_on_last_roundtrip(mut self, value: bool) -> Self {
        self.on_last_roundtrip = value;
        self
    }

    pub fn stop_on_signal(mut self, receiver: CancellationSignal) -> Self {
        self.signal_receiver = Some(receiver);
        self
    }

    pub fn build(self) -> StopSettings {
        StopSettings {
            on_last_roundtrip: self.on_last_roundtrip,
            signal_receiver: self.signal_receiver,
        }
    }
}

impl Scheduler {
    pub fn new(mainloop: MainLoopRc, core: CoreRc) -> Self {
        Self {
            mainloop,
            core,
            sync_seq: Default::default(),
            done: Default::default(),
        }
    }

    #[instrument(skip(self))]
    pub fn schedule_roundtrip(&self) {
        tracing::trace!("sending sync");
        self.sync_seq.replace(Some(self.core.sync(0).unwrap()));
    }

    pub fn stop(&self) {
        self.mainloop.quit();
        self.done.set(true);
    }

    #[instrument(skip(self))]
    pub fn run(self, stop_settings: StopSettings) {
        let mut listener_core = None;
        if stop_settings.on_last_roundtrip {
            self.schedule_roundtrip();
            listener_core = Some(
                self.core
                    .add_listener_local()
                    .done({
                        let done = self.done.clone();
                        let mainloop = self.mainloop.clone();
                        let sync_seq = self.sync_seq.clone();
                        move |id, seq| {
                            tracing::trace!("sync");
                            if id == PW_ID_CORE && Some(seq) == sync_seq.get() {
                                tracing::trace!("core sync");
                                done.set(true);
                                mainloop.quit();
                            }
                        }
                    })
                    .register(),
            );
        }

        let attached_receiver = stop_settings.signal_receiver.map(|stop_signal_receiver| {
            stop_signal_receiver.attach(self.mainloop.loop_(), {
                let done = self.done.clone();
                let mainloop = self.mainloop.clone();
                move || {
                    done.set(true);
                    mainloop.quit()
                }
            })
        });

        tracing::trace!("starting pipewire loop");
        while !self.done.get() {
            self.mainloop.run();
        }
        tracing::trace!("pipewire loop finished");

        if let Some(attached_receiver) = attached_receiver {
            attached_receiver.deattach();
        }

        drop(listener_core);
    }
}
