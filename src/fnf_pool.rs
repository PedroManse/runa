use crate::fnf::{FnFEventLoopError, FnFRunner};
use crate::{Command, CommandRunner};
use crossbeam_channel as mpmc;
use std::any::Any;
use std::thread::JoinHandle;

type MR<Cmd> = mpmc::Receiver<Cmd>;
type PoolRunner<Cmd> = FnFRunner<Cmd, MR<Cmd>>;

/// API of [`FnFRunner`] for managing multiple runners
pub struct PoolFnFAPI<Cmd, const N: usize>
where
    Cmd: Command<Result = ()>,
{
    send_cmd: mpmc::Sender<Cmd>,
    runners: [JoinHandle<Result<PoolRunner<Cmd>, FnFEventLoopError>>; N],
}

#[derive(Debug)]
pub enum PoolFnFCloseError<Cmd>
where
    Cmd: Command,
{
    Send(mpmc::SendError<Cmd>),
    Join(Box<dyn Any + Send>),
}

impl<Cmd, const N: usize> CommandRunner for PoolFnFAPI<Cmd, N>
where
    Cmd: Command<Result = ()>,
{
    type Cmd = Cmd;
    type SendAck = Result<(), mpmc::SendError<Cmd>>;
    type CloseResult =
        Result<[Result<PoolRunner<Cmd>, FnFEventLoopError>; N], mpmc::SendError<Cmd>>;
    unsafe fn new() -> Self {
        let (tx_cmd, rx_cmd) = mpmc::unbounded();
        let runners = [(); N].map(|()| FnFRunner::spawn(rx_cmd.clone()));
        Self {
            send_cmd: tx_cmd,
            runners,
        }
    }
    fn send(&self, cmd: Self::Cmd) -> Self::SendAck {
        self.send_cmd.send(cmd)
    }
    fn close_with(self, mut s: impl crate::StopRunner<Self::Cmd>) -> Self::CloseResult {
        for _ in 0..self.runners.len() {
            self.send(s.get())?;
        }
        Ok(self
            .runners
            .map(std::thread::JoinHandle::join)
            .map(|e| e.map_err(FnFEventLoopError::ThreadPanic)?))
    }
}
