use crate::{ActionResult, CmdRst, Command, CommandRunner};
use std::any::Any;
use std::sync::mpsc::{self, Receiver, RecvError, SendError, Sender};
use std::thread::JoinHandle;

pub struct QueueRunner<Cmd>
where
    Cmd: Command,
{
    recv_cmd: Receiver<Cmd>,
    send_res: Sender<CmdRst<Cmd>>,
}

impl<Cmd> QueueRunner<Cmd>
where
    Cmd: Command,
{
    fn get(&self) -> Result<Cmd, RecvError> {
        self.recv_cmd.recv()
    }
    fn send(&self, res: CmdRst<Cmd>) -> Result<(), SendError<CmdRst<Cmd>>> {
        self.send_res.send(res)
    }
    fn exec(cmd: Cmd) -> ActionResult<Cmd::Result> {
        cmd.execute()
    }
    fn spawn(recv_cmd: Receiver<Cmd>, send_res: Sender<CmdRst<Cmd>>) -> JoinHandle<Self> {
        std::thread::spawn(|| {
            let runner = QueueRunner { recv_cmd, send_res };
            loop {
                let cmd = runner.get().unwrap();
                let r = QueueRunner::exec(cmd);
                let ActionResult::Normal(res) = r else { break };
                runner.send(res).unwrap();
            }
            runner
        })
    }
}

pub struct QueueAPI<Cmd>
where
    Cmd: Command,
{
    send_cmd: Sender<Cmd>,
    recv_res: Receiver<CmdRst<Cmd>>,
    thread: JoinHandle<QueueRunner<Cmd>>,
}

#[derive(Debug)]
pub enum QueueCloseError<Cmd>
where
    Cmd: Command,
{
    Send(SendError<Cmd>),
    Join(Box<dyn Any + Send>),
}

impl<Cmd> CommandRunner for QueueAPI<Cmd>
where
    Cmd: Command,
{
    type Cmd = Cmd;
    type SendAck = Result<(), SendError<Cmd>>;
    type CloseResult = Result<QueueRunner<Cmd>, QueueCloseError<Cmd>>;
    fn new() -> Self {
        let (send_cmd, recv_cmd) = mpsc::channel();
        let (send_res, recv_res) = mpsc::channel();
        let thread = QueueRunner::spawn(recv_cmd, send_res);
        QueueAPI {
            send_cmd,
            recv_res,
            thread,
        }
    }
    fn send(&self, cmd: Self::Cmd) -> Self::SendAck {
        self.send_cmd.send(cmd)
    }
    fn close_with(self, s: impl crate::StopRunner<Self::Cmd>) -> Self::CloseResult {
        let cmd = s.get();
        self.send_cmd.send(cmd).map_err(QueueCloseError::Send)?;
        self.thread.join().map_err(QueueCloseError::Join)
    }
}

impl<Cmd> QueueAPI<Cmd>
where
    Cmd: Command,
{
    pub fn recv(&self) -> Result<CmdRst<Cmd>, RecvError> {
        self.recv_res.recv()
    }
    pub fn try_recv(&self) -> Result<CmdRst<Cmd>, mpsc::TryRecvError> {
        self.recv_res.try_recv()
    }
}
