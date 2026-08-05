use crate::{ActionResult, ChanRecv, Command};
use std::fmt;
use std::marker::PhantomData;
use std::thread::JoinHandle;

/// Runner that executes a command
pub struct FnFRunner<Cmd, R>
where
    Cmd: Command<Result = ()>,
    R: ChanRecv<Cmd>,
{
    pub(crate) d: PhantomData<Cmd>,
    pub(crate) recv_cmd: R,
}

#[derive(Debug)]
pub enum FnFEventLoopError {
    SendErr,
    RecvErr,
    ThreadPanic(Box<dyn std::any::Any + Send>),
}

impl fmt::Display for FnFEventLoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecvErr => write!(f, "Failed to recieve"),
            Self::SendErr => write!(f, "Failed to write"),
            Self::ThreadPanic(..) => write!(f, "Worker panicked"),
        }
    }
}

impl std::error::Error for FnFEventLoopError {}

impl<Cmd, R> FnFRunner<Cmd, R>
where
    Cmd: Command<Result = ()>,
    R: ChanRecv<Cmd>,
{
    /// # Errors
    /// Will fail if request can't be received
    pub(crate) fn get(&self) -> Result<Cmd, R::Err> {
        self.recv_cmd.recv_t()
    }
    pub(crate) fn exec(cmd: Cmd) -> ActionResult<Cmd> {
        cmd.execute()
    }
}

impl<Cmd, R> FnFRunner<Cmd, R>
where
    Cmd: Command<Result = ()>,
    R: ChanRecv<Cmd> + Send + 'static,
    <R as ChanRecv<Cmd>>::Err: std::fmt::Debug,
{
    /// # Panics
    /// The default runners panic if the channels they're bound to are dropped.
    pub(crate) fn spawn(recv_cmd: R) -> JoinHandle<Result<Self, FnFEventLoopError>> {
        std::thread::spawn(|| {
            let runner = Self {
                recv_cmd,
                d: PhantomData,
            };
            loop {
                let cmd = runner.get().map_err(|_| FnFEventLoopError::RecvErr)?;
                let r = Self::exec(cmd);
                let ActionResult::Normal(()) = r else { break };
            }
            Ok(runner)
        })
    }
}
