use std::future::*;
use std::pin::pin;
use std::task::*;

const PENDING_SLEEP_MS: u64 = 10_000;

unsafe fn rwclone(_p: *const ()) -> RawWaker {
    make_raw_waker()
}
unsafe fn rwwake(_p: *const ()) {}
unsafe fn rwwakebyref(_p: *const ()) {}
unsafe fn rwdrop(_p: *const ()) {}

static VTABLE: RawWakerVTable = RawWakerVTable::new(rwclone, rwwake, rwwakebyref, rwdrop);

fn make_raw_waker() -> RawWaker {
    static DATA: () = ();
    RawWaker::new(&DATA, &VTABLE)
}

pub(crate) fn busy_loop<T>(future: impl Future<Output = T>) -> T {
    let mut pin = pin!(future);
    let waker = unsafe { Waker::from_raw(make_raw_waker()) };
    let mut ctx = Context::from_waker(&waker);
    loop {
        match pin.as_mut().poll(&mut ctx) {
            Poll::Ready(x) => {
                return x;
            }
            Poll::Pending => std::thread::sleep(std::time::Duration::from_millis(PENDING_SLEEP_MS)),
        }
    }
}
