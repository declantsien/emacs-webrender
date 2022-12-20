use std::ptr;

use libc::pselect;
use lisp_types::bindings::thread_select;

use crate::event_loop::{FdSet, Timespec};

pub async fn tokio_select_fds(
    nfds: i32,
    readfds: &FdSet,
    writefds: &FdSet,
    timeout: &Timespec,
) -> i32 {
    unsafe {
        thread_select(
            Some(pselect),
            nfds,
            readfds.0,
            writefds.0,
            ptr::null_mut(),
            timeout.0,
            ptr::null_mut(),
        )
    }
}
