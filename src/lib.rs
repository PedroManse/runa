pub mod oneshot;
pub mod queue;

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

pub trait SimpleStop: Command {
    fn make_stop_command() -> Self;
}
struct SimpleCloser;

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
    fn close_with(self, s: impl StopRunner<Self::Cmd>) -> Self::CloseResult;
    fn close(self) -> Self::CloseResult
    where
        Self::Cmd: SimpleStop,
        Self: Sized,
    {
        self.close_with(SimpleCloser)
    }

    /// No need to remember to .close the runner if you use scope
    fn scope_with(closer: impl StopRunner<Self::Cmd>, f: impl Fn(&Self)) -> Self::CloseResult
    where
        Self: Sized,
    {
        let runner = Self::new();
        f(&runner);
        runner.close_with(closer)
    }

    fn scope(f: impl Fn(&Self)) -> Self::CloseResult
    where
        Self: Sized,
        Self::Cmd: SimpleStop,
    {
        let runner = Self::new();
        f(&runner);
        runner.close()
    }
}

impl<C> StopRunner<C> for SimpleCloser
where
    C: SimpleStop,
{
    fn get(&self) -> C {
        C::make_stop_command()
    }
}

pub type CmdRst<C> = <C as Command>::Result;
