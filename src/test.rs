use crate as supera;
use supera::CommandRunner;

#[derive(Debug, Clone, Copy)]
pub enum MathAction {
    Sub(i32, i32),
    Stop,
}

impl supera::SimpleStop for MathAction {
    fn make_stop_command() -> Self {
        MathAction::Stop
    }
}

impl supera::Command for MathAction {
    type Result = i32;
    fn execute(self) -> supera::ActionResult<Self::Result> {
        supera::ActionResult::Normal(match self {
            Self::Sub(a, b) => a - b,
            Self::Stop => return supera::ActionResult::Stop,
        })
    }
}

mod queue {
    use super::*;
    #[test]
    fn single_values() {
        const COUNT: usize = 500_000;
        let mut outs = Vec::with_capacity(COUNT);
        supera::queue_single::SingleQueueAPI::<MathAction>::scope(|q| {
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
    fn pool_values() {
        const COUNT: usize = 500_000;
        let mut outs = Vec::with_capacity(COUNT);
        let rs = supera::queue_pool::PoolQueueAPI::<MathAction, 2>::scope(|q| {
            let ma = MathAction::Sub(2, 1);
            for _ in 0..COUNT {
                q.send(ma).unwrap();
            }
            for _ in 0..COUNT {
                outs.push(q.recv().unwrap());
            }
        })
        .unwrap();
        assert_eq!(outs, vec![1; COUNT]);
        for r in rs {
            r.unwrap();
        }
    }

    #[test]
    fn single_manual_close() {
        let rs = unsafe { supera::queue_single::SingleQueueAPI::new() };
        rs.send(MathAction::Sub(3, 2)).unwrap();
        rs.recv().unwrap();
        rs.close().unwrap();
    }

    #[test]
    fn pool_manual_close() {
        let rs = unsafe { supera::queue_pool::PoolQueueAPI::<MathAction, 3>::new() };
        rs.send(MathAction::Sub(3, 2)).unwrap();
        rs.recv().unwrap();
        for r in rs.close().unwrap() {
            r.unwrap();
        }
    }
}

mod oneshot {
    use super::*;
    #[test]
    fn single_values() {
        const COUNT: usize = 5_000;
        supera::oneshot_single::OneShotAPI::scope(|q| {
            for _ in 0..COUNT {
                let ma = MathAction::Sub(2, 1);
                let mr = q.send(ma).unwrap();
                let r = mr.recv().unwrap();
                assert_eq!(r, 1);
            }
        })
        .unwrap();
    }

    #[test]
    fn pool_values() {
        const COUNT: usize = 50_000;
        let runners = supera::oneshot_pool::OneShotPoolAPI::<MathAction, 10>::scope(|q| {
            for _ in 0..COUNT {
                let ma = MathAction::Sub(2, 1);
                let mr = q.send(ma).unwrap();
                let r = mr.recv().unwrap();
                assert_eq!(r, 1);
            }
        })
        .unwrap();
        for r in runners {
            r.unwrap();
        }
    }

    #[test]
    fn single_manual_close() {
        use supera::oneshot_single::OneShotAPI;
        const COUNT: usize = 2_500;
        let q = unsafe { OneShotAPI::new() };
        for _ in 0..COUNT {
            let ma = MathAction::Sub(2, 1);
            let mr = q.send(ma).unwrap();
            let r = mr.recv().unwrap();
            assert_eq!(r, 1);
        }
        q.close().unwrap();
    }

    #[test]
    fn pool_manual_close() {
        use supera::oneshot_pool::OneShotPoolAPI;
        const COUNT: usize = 2_500;
        let q = unsafe { OneShotPoolAPI::<MathAction, 3>::new() };
        for _ in 0..COUNT {
            let ma = MathAction::Sub(2, 1);
            let mr = q.send(ma).unwrap();
            let r = mr.recv().unwrap();
            assert_eq!(r, 1);
        }
        for r in q.close().unwrap() {
            r.unwrap();
        }
    }
}
