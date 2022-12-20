use std::ptr;

use futures::Future;
use libc::pselect;
use lisp_types::bindings::thread_select;

use crate::event_loop::{FdSet, Timespec};

async fn async_thread_select(
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

pub fn tokio_select_fds<'a>(
    nfds: i32,
    readfds: &'a FdSet,
    writefds: &'a FdSet,
    timeout: &'a Timespec,
) -> impl Future<Output = i32> + 'a {
    async move {
        let nfds: i32 = async_thread_select(nfds, readfds, writefds, timeout).await;
        nfds
    }
}
