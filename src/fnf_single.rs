use crate::fnf::{FnFEventLoopError, FnFRunner};
use crate::{Command, CommandRunner};
use std::any::Any;
use std::fmt;
use std::sync::mpsc;
use std::thread::JoinHandle;

type SR<Cmd> = mpsc::Receiver<Cmd>;
type RunnerResult<Cmd> = Result<FnFRunner<Cmd, SR<Cmd>>, FnFEventLoopError>;

/// API of [`FnFRunner`] for managing a single runner
pub struct SingleFnFAPI<Cmd>
where
    Cmd: Command<Result = ()>,
{
    send_cmd: mpsc::Sender<Cmd>,
    thread: JoinHandle<RunnerResult<Cmd>>,
}

#[derive(Debug)]
pub enum SingleFnFCloseError<Cmd>
where
    Cmd: Command,
{
    Send(mpsc::SendError<Cmd>),
    Join(Box<dyn Any + Send>),
    Worker(FnFEventLoopError),
}

impl<Cmd: Command> fmt::Display for SingleFnFCloseError<Cmd> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Join(e) => write!(f, "Failed to join thread, {e:?}"),
            Self::Send(cmd) => write!(f, "Failed to send command {cmd}"),
            Self::Worker(e) => write!(f, "Worker failed with: {e}"),
        }
    }
}

impl<Cmd: fmt::Debug + Command> std::error::Error for SingleFnFCloseError<Cmd> {}

impl<Cmd> CommandRunner for SingleFnFAPI<Cmd>
where
    Cmd: Command<Result = ()>,
{
    type Cmd = Cmd;
    type SendAck = Result<(), mpsc::SendError<Cmd>>;
    type CloseResult = Result<FnFRunner<Cmd, SR<Cmd>>, SingleFnFCloseError<Cmd>>;
    unsafe fn new() -> Self {
        let (send_cmd, recv_cmd) = mpsc::channel();
        let thread = FnFRunner::spawn(recv_cmd);
        SingleFnFAPI { send_cmd, thread }
    }
    fn send(&self, cmd: Self::Cmd) -> Self::SendAck {
        self.send_cmd.send(cmd)
    }
    fn close_with(self, mut s: impl crate::StopRunner<Self::Cmd>) -> Self::CloseResult {
        let cmd = s.get();
        self.send_cmd.send(cmd).map_err(SingleFnFCloseError::Send)?;
        self.thread
            .join()
            .map_err(SingleFnFCloseError::Join)?
            .map_err(SingleFnFCloseError::Worker)
    }
}
