use crate as runa;
use runa::CommandRunner;

#[derive(Debug, Clone, Copy)]
pub enum MathAction {
    Sub(i32, i32),
    Stop,
}

impl runa::SimpleStop for MathAction {
    fn make_stop_command() -> Self {
        MathAction::Stop
    }
}

impl runa::Command for MathAction {
    type Result = i32;
    fn execute(self) -> runa::ActionResult<Self::Result> {
        runa::ActionResult::Normal(match self {
            Self::Sub(a, b) => a - b,
            Self::Stop => return runa::ActionResult::Stop,
        })
    }
}

#[test]
fn pooled_queue_values() {
    const COUNT: usize = 50_000;
    let mut outs = Vec::with_capacity(COUNT);
    let rs = runa::queue_pool::PoolQueueAPI::<MathAction, 2>::scope(|q| {
        let ma = MathAction::Sub(2, 1);
        for _ in 0..COUNT {
            q.send(ma).unwrap();
        }
        for _ in 0..COUNT {
            outs.push(q.recv().unwrap());
        }
        std::thread::yield_now();
    })
    .unwrap();
    for r in rs {
        r.unwrap();
    }
    assert_eq!(outs, vec![1; COUNT]);
}

#[test]
fn single_queue_values() {
    const COUNT: usize = 50_000;
    let mut outs = Vec::with_capacity(COUNT);
    runa::queue_single::SingleQueueAPI::<MathAction>::scope(|q| {
        let ma = MathAction::Sub(2, 1);
        for _ in 0..COUNT {
            q.send(ma).unwrap();
        }
        for _ in 0..COUNT {
            outs.push(q.recv().unwrap());
        }
        std::thread::yield_now();
    })
    .unwrap();
    assert_eq!(outs, vec![1; COUNT]);
}

#[test]
fn oneshot_values() {
    const COUNT: usize = 5_000;
    runa::oneshot::OneShotAPI::scope(|q| {
        for _ in 0..COUNT {
            let ma = MathAction::Sub(2, 1);
            let mr = q.send(ma).unwrap();
            let r = mr.recv().unwrap();
            assert_eq!(r, 1);
        }
    })
    .unwrap();
}

fn f1(q: &runa::oneshot::OneShotAPI<MathAction>, c: usize) {
    for _ in 0..c {
        let ma = MathAction::Sub(2, 1);
        let mr = q.send(ma).unwrap();
        let r = mr.recv().unwrap();
        assert_eq!(r, 1);
    }
}

fn f2(q: &runa::oneshot::OneShotAPI<MathAction>, c: usize) {
    for _ in 0..c {
        let ma = MathAction::Sub(2, 1);
        let mr = q.send(ma).unwrap();
        let r = mr.recv().unwrap();
        assert_eq!(r, 1);
    }
}

#[test]
fn oneshot_nested() {
    use runa::oneshot::OneShotAPI;
    const COUNT: usize = 2_500;
    OneShotAPI::scope(|q| {
        f1(q, COUNT);
        f2(q, COUNT);
    })
    .unwrap();
}

#[test]
fn oneshot_manual_close() {
    use runa::oneshot::OneShotAPI;
    const COUNT: usize = 2_500;
    let q = OneShotAPI::new();
    f1(&q, COUNT);
    f2(&q, COUNT);
    q.close().unwrap();
}
