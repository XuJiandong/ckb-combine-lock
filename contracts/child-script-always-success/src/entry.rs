use crate::error::Error;
use ckb_combine_lock_common::chained_exec::continue_running;
use ckb_combine_lock_common::log;
use ckb_std::env::argv;
use core::result::Result;

pub fn main() -> Result<(), Error> {
    inner_main()?;

    continue_running(argv()).map_err(|_| Error::ChainedExec)?;
    Ok(())
}

pub fn inner_main() -> Result<(), Error> {
    // always success
    log!("always success!");
    Ok(())
}
