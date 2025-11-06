use std::time::Duration;

use runa::{ActionResult, Command, CommandRunner, SimpleStop};

#[derive(Debug, Clone, Copy)]
pub enum MathAction {
    Sum(i32, i32),
    Sub(i32, i32),
    Stop,
}

impl SimpleStop for MathAction {
    fn make_stop_command() -> Self {
        MathAction::Stop
    }
}

impl Command for MathAction {
    type Result = i32;
    fn execute(self) -> runa::ActionResult<Self::Result> {
        std::thread::sleep(Duration::from_secs(5));
        ActionResult::Normal(match self {
            Self::Sum(a, b) => a + b,
            Self::Sub(a, b) => a - b,
            Self::Stop => return ActionResult::Stop,
        })
    }
}

fn main() {
    runa::oneshot::OneShotAPI::scope(|math_runner| {
        runa::oneshot::OneShotAPI::scope(|math_runner2| {
            let ma = MathAction::Sum(3, 5);
            let r0 = math_runner.send(ma).unwrap();
            let r1 = math_runner.send(ma).unwrap();
            let r2 = math_runner.send(ma).unwrap();
            let r3 = math_runner.send(ma).unwrap();
            let r4 = math_runner.send(ma).unwrap();

            let r5 = math_runner2.send(ma).unwrap();
            let r6 = math_runner2.send(ma).unwrap();
            let r7 = math_runner2.send(ma).unwrap();
            let r8 = math_runner2.send(ma).unwrap();
            let r9 = math_runner2.send(ma).unwrap();
            for r in [r0, r5, r1, r6, r2, r7, r3, r8, r4, r9] {
                dbg!(r.recv().unwrap());
            }
        })
        .unwrap();
    })
    .unwrap();
}
