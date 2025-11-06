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
        let x = match self {
            Self::Sum(a, b) => a + b,
            Self::Sub(a, b) => a - b,
            Self::Stop => return ActionResult::Stop,
        };
        ActionResult::Normal(x)
    }
}

fn main() {
    let rs = runa::queue_pool::PoolQueueAPI::<MathAction, 9>::scope(|q| {
        let ma = MathAction::Sub(2, 1);
        q.send(ma).unwrap();
        q.send(ma).unwrap();
        q.send(ma).unwrap();
        q.send(ma).unwrap();
        q.send(ma).unwrap();
        q.send(ma).unwrap();
        q.send(ma).unwrap();
        q.send(ma).unwrap();
        q.send(ma).unwrap();
        for _ in 0..9 {
            dbg!(q.recv().unwrap());
        }
        std::thread::yield_now();
    })
    .unwrap();
    for r in rs {
        r.unwrap();
    }
}
