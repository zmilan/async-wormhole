use std::io::Error;
use std::thread::LocalKey;
use std::cell::Cell;

use crossbeam::queue::{ArrayQueue, PopError};
use switcheroo::stack::*;

use super::{ AsyncWormhole, AsyncYielder };

pub struct OneMbAsyncPool {
    pool: ArrayQueue<OneMbStack>
}

impl OneMbAsyncPool {
    pub fn new(capacity: usize) -> Self {
        Self { pool: ArrayQueue::new(capacity)}
    }

    pub fn with_tls<'a, F, Output, TLS>(&mut self, tls: &'static LocalKey<Cell<*const TLS>>, f: F)
        -> Result<AsyncWormhole<'a, OneMbStack, Output, TLS>, Error>
    where
        F: FnOnce(AsyncYielder<Output>) -> Output + 'a,
    {
        match self.pool.pop() {
            Err(PopError) => {
                let stack = OneMbStack::new()?;
                let mut wormhole = AsyncWormhole::new(stack, f)?;
                wormhole.preserve_tls(tls);
                Ok(wormhole)
            },
            Ok(stack) => {
                let mut wormhole = AsyncWormhole::new(stack, f)?;
                wormhole.preserve_tls(tls);
                Ok(wormhole)
            }
        }
    }

    pub fn recycle<Output, TLS>(&mut self, async_wormhole: AsyncWormhole<OneMbStack, Output, TLS>) {
        let _ = self.pool.push(async_wormhole.stack());
    }
}