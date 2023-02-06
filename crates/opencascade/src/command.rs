use crate::Error;
use cxx::memory::UniquePtrTarget;
use cxx::UniquePtr;
use opencascade_sys::ffi::{Message_ProgressRange, Message_ProgressRange_ctor};
use std::pin::Pin;

pub trait Command {
    fn is_done(&self) -> bool;
    fn build(&mut self, progress: &Message_ProgressRange);
}

pub trait Builder<T>: Command {
    unsafe fn value(&mut self) -> &T;

    fn try_build(&mut self) -> Result<&T, Error> {
        if !self.is_done() {
            let progress = Message_ProgressRange_ctor();
            self.build(&progress);

            if !self.is_done() {
                return Err("opencascade command failed".into());
            }
        }
        // SAFETY: safe since is_done and build were checked
        Ok(unsafe { self.value() })
    }
}

pub trait PinCommand {
    fn is_done(&self) -> bool;
    fn build(self: Pin<&mut Self>, progress: &Message_ProgressRange);
}

pub trait PinBuilder<T>: PinCommand {
    unsafe fn value(self: Pin<&mut Self>) -> &T;

    fn try_build(pin: &mut UniquePtr<Self>) -> Result<&T, Error>
    where
        Self: Sized,
        Self: UniquePtrTarget,
    {
        if !pin.is_done() {
            let progress = Message_ProgressRange_ctor();
            pin.pin_mut().build(&progress);
        }

        // SAFETY: safe since is_done and build were checked
        Ok(unsafe { pin.pin_mut().value() })
    }
}
