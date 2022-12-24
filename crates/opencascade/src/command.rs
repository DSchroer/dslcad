use crate::Error;
use opencascade_sys::ffi::{Message_ProgressRange, Message_ProgressRange_ctor};

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
