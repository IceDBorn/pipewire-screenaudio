use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

use pipewire::{
    channel::{channel, AttachedReceiver, Receiver, Sender},
    loop_::Loop,
};

#[derive(Debug)]
pub struct TerminateSignal;

pub struct CancellationSignal {
    receiver: Receiver<TerminateSignal>,
}

pub struct AttachedCancellationSignal<'a> {
    receiver: AttachedReceiver<'a, TerminateSignal>,
}

#[derive(Clone)]
pub struct CancellationController {
    sender: Sender<TerminateSignal>,
}

impl CancellationSignal {
    pub fn pair() -> (CancellationController, CancellationSignal) {
        let (sender, receiver) = channel();
        (
            CancellationController { sender },
            CancellationSignal { receiver },
        )
    }

    pub fn attach(
        self,
        loop_: &Loop,
        callback: impl Fn() + 'static,
    ) -> AttachedCancellationSignal<'_> {
        AttachedCancellationSignal {
            receiver: self.receiver.attach(loop_, move |_| callback()),
        }
    }
}

impl<'a> AttachedCancellationSignal<'a> {
    pub fn deattach(self) -> CancellationSignal {
        CancellationSignal {
            receiver: self.receiver.deattach(),
        }
    }
}

pub struct TimeoutHandle {
    handle: Option<JoinHandle<()>>,
}

impl Drop for TimeoutHandle {
    fn drop(&mut self) {
        let handle = self
            .handle
            .take()
            .expect("handle should always have a value");
        handle.thread().unpark();
        handle.join().unwrap();
    }
}

impl CancellationController {
    pub fn cancel(&self) {
        if self.sender.send(TerminateSignal).is_err() {
            tracing::warn!("error while trying to cancel pipewire loop");
        }
    }

    pub fn timeout(&self, duration: Duration) -> TimeoutHandle {
        let handle = thread::spawn({
            let cloned_self = self.clone();
            move || {
                thread::park_timeout(duration);
                cloned_self.cancel();
            }
        });
        TimeoutHandle {
            handle: Some(handle),
        }
    }
}
