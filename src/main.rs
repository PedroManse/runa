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
        std::thread::sleep(Duration::from_secs(1));
        ActionResult::Normal(match self {
            Self::Sum(a, b) => a + b,
            Self::Sub(a, b) => a - b,
            Self::Stop => return ActionResult::Stop,
        })
    }
}

fn main() {
    runa::queue::QueueAPI::scope(|q1| {
        runa::queue::QueueAPI::scope(|q2| {
            let ma = MathAction::Sum(3, 5);
            q1.send(ma).unwrap();
            q1.send(ma).unwrap();
            q2.send(ma).unwrap();
            q1.send(ma).unwrap();
            q2.send(ma).unwrap();
            q1.send(ma).unwrap();
            q2.send(ma).unwrap();
            q1.send(ma).unwrap();
            q2.send(ma).unwrap();
            for _ in 0..4 {
                dbg!(q1.recv().unwrap());
                dbg!(q2.recv().unwrap());
            }
            std::thread::yield_now();
        })
        .unwrap();
    })
    .unwrap();
}
