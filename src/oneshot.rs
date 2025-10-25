use std::any::Any;
use std::fmt::Debug;
use std::sync::mpsc;
use std::thread::JoinHandle;

use crate::{ActionResult, CmdRst, Command, CommandRunner};

type InternalCommandLink<Cmd> = oneshot::Sender<CmdRst<Cmd>>;
type ExternalCommandLink<Cmd> = oneshot::Receiver<CmdRst<Cmd>>;

pub struct QueuedCommand<Cmd>
where
    Cmd: Command,
{
    cmd: Cmd,
    chan: InternalCommandLink<Cmd>,
}

impl<Cmd> Debug for QueuedCommand<Cmd>
where
    Cmd: Debug + Command,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Queued Command ({:?})", self.cmd)
    }
}

#[derive(Debug)]
pub struct OneShotRunner<Cmd>
where
    Cmd: Command,
{
    reqs: mpsc::Receiver<QueuedCommand<Cmd>>,
}

pub struct OneShotAPI<Cmd>
where
    Cmd: Command,
{
    cmd_queue: mpsc::Sender<QueuedCommand<Cmd>>,
    thread: JoinHandle<OneShotRunner<Cmd>>,
}

impl<Cmd> OneShotRunner<Cmd>
where
    Cmd: Command,
    <Cmd as Command>::Result: Debug,
{
    fn get(&self) -> Result<QueuedCommand<Cmd>, mpsc::RecvError> {
        self.reqs.recv()
    }
    fn exec(cmd: Cmd) -> ActionResult<Cmd::Result> {
        cmd.execute()
    }
    fn spawn(rx: mpsc::Receiver<QueuedCommand<Cmd>>) -> JoinHandle<Self> {
        std::thread::spawn(move || {
            let runner = OneShotRunner::<Cmd> { reqs: rx };
            loop {
                let msg = runner.get().unwrap();
                let r = OneShotRunner::exec(msg.cmd);
                let ActionResult::Normal(res) = r else { break };
                msg.chan.send(res).unwrap();
            }
            runner
        })
    }
}

#[derive(Debug)]
pub enum OneShotCloseError<Cmd>
where
    Cmd: Command,
{
    SendError(QueuedCommand<Cmd>),
    JoinError(Box<dyn Any + Send>),
}

impl<Cmd> CommandRunner for OneShotAPI<Cmd>
where
    Cmd: Command,
    <Cmd as Command>::Result: Debug,
{
    type Cmd = Cmd;
    type SendAck = Result<ExternalCommandLink<Cmd>, QueuedCommand<Cmd>>;
    type CloseResult = Result<OneShotRunner<Cmd>, OneShotCloseError<Cmd>>;
    fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let thread = OneShotRunner::spawn(rx);
        OneShotAPI {
            cmd_queue: tx,
            thread,
        }
    }
    fn send(&self, cmd: Self::Cmd) -> Self::SendAck {
        let (tx, rx) = oneshot::channel();
        let msg = QueuedCommand { cmd, chan: tx };
        self.cmd_queue.send(msg).map_err(|e| e.0)?;
        Ok(rx)
    }
    fn close(self, c: impl crate::StopRunner<Self::Cmd>) -> Self::CloseResult {
        self.send(c.get()).map_err(OneShotCloseError::SendError)?;
        self.thread.join().map_err(OneShotCloseError::JoinError)
    }
}
