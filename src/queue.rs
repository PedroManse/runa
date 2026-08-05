use crate::{ActionResult, ChanRecv, ChanSend, CmdRst, Command};
use std::fmt;
use std::marker::PhantomData;
use std::thread::JoinHandle;

/// Runner that sends responses to a queue
pub struct QueueRunner<Cmd, Sc, R, S>
where
    Cmd: Command,
    Sc: ChanSend<Cmd> + Send + 'static,
    R: ChanRecv<Cmd>,
    S: ChanSend<CmdRst<Cmd>>,
{
    pub(crate) d: PhantomData<Cmd>,
    pub(crate) send_cmd: Sc,
    pub(crate) recv_cmd: R,
    pub(crate) send_res: S,
}

#[derive(Debug)]
pub enum QueueEventLoopError {
    SendErr,
    RecvErr,
    ThreadPanic(Box<dyn std::any::Any + Send>),
}

impl fmt::Display for QueueEventLoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecvErr => write!(f, "Failed to recieve"),
            Self::SendErr => write!(f, "Failed to write"),
            Self::ThreadPanic(..) => write!(f, "Worker panicked"),
        }
    }
}

impl std::error::Error for QueueEventLoopError {}

impl<Cmd, Sc, S, R> QueueRunner<Cmd, Sc, R, S>
where
    Cmd: Command,
    Sc: ChanSend<Cmd> + Send + 'static,
    R: ChanRecv<Cmd>,
    S: ChanSend<CmdRst<Cmd>>,
{
    /// # Errors
    /// Will fail if request can't be received
    pub(crate) fn get(&self) -> Result<Cmd, R::Err> {
        self.recv_cmd.recv_t()
    }
    /// # Errors
    /// Will fail if response can't be sent
    pub(crate) fn send(&self, res: CmdRst<Cmd>) -> Result<(), S::Err> {
        self.send_res.send_t(res)
    }
    pub(crate) fn exec(cmd: Cmd) -> ActionResult<Cmd> {
        cmd.execute()
    }
}

impl<Cmd, Sc, R, S> QueueRunner<Cmd, Sc, R, S>
where
    Cmd: Command,
    Sc: ChanSend<Cmd> + Send + 'static,
    R: ChanRecv<Cmd> + Send + 'static,
    S: ChanSend<CmdRst<Cmd>> + Send + 'static,
    <Sc as ChanSend<Cmd>>::Err: std::fmt::Debug,
    //<R as ChanRecv<Cmd>>::Err: std::fmt::Debug,
    //<S as ChanSend<Cmd::Result>>::Err: std::fmt::Debug,
{
    /// # Panics
    /// The default runners panic if the channels they're bound to are dropped.
    pub(crate) fn spawn(
        send_cmd: Sc,
        recv_cmd: R,
        send_res: S,
    ) -> JoinHandle<Result<Self, QueueEventLoopError>> {
        std::thread::spawn(move || {
            let runner = Self {
                send_cmd,
                recv_cmd,
                send_res,
                d: PhantomData,
            };
            loop {
                let cmd = runner.get().map_err(|_| QueueEventLoopError::RecvErr)?;
                match Self::exec(cmd) {
                    ActionResult::Stop => break,
                    ActionResult::Next(cmd) => runner.send_cmd.send_t(cmd).unwrap(),
                    ActionResult::Normal(res) => {
                        runner.send(res).map_err(|_| QueueEventLoopError::SendErr)?;
                    }
                }
            }
            Ok(runner)
        })
    }
}
