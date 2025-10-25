pub mod oneshot;

#[derive(Debug)]
pub enum ActionResult<R> {
    Normal(R),
    Stop,
}

pub trait Command: Send + Sync + 'static {
    type Result: Send;
    fn execute(self) -> ActionResult<Self::Result>;
}

/// Creates a command that would halt the command runner>
pub trait StopRunner<C: Command> {
    fn get(&self) -> C;
}

/// A command runner's API
pub trait CommandRunner {
    /// The command it accepts
    type Cmd: Command;
    /// Result of sending a command to a runner
    type SendAck;
    /// The result of halting a runner
    type CloseResult;

    fn new() -> Self;
    fn send(&self, cmd: Self::Cmd) -> Self::SendAck;
    fn close(self, s: impl StopRunner<Self::Cmd>) -> Self::CloseResult;

    /// No need to remember to .close the runner if you use scope
    fn scope(closer: impl StopRunner<Self::Cmd>, f: impl Fn(&Self)) -> Self::CloseResult
    where
        Self: Sized,
    {
        let runner = Self::new();
        f(&runner);
        runner.close(closer)
    }
}

pub type CmdRst<C> = <C as Command>::Result;
