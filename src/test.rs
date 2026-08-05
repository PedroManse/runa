#![allow(
    clippy::unwrap_used,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]
use crate as supera;
use supera::CommandRunner;

#[derive(Debug, Clone, Copy)]
pub enum MathAction {
    Sub(i32, i32),
    Stop,
}

#[derive(Debug, Clone, Copy)]
pub enum FnFMathAction {
    Sub,
    Stop,
}

impl supera::SimpleStop for MathAction {
    fn make_stop_command() -> Self {
        MathAction::Stop
    }
}

impl supera::Command for MathAction {
    type Result = i32;
    fn execute(self) -> supera::ActionResult<MathAction> {
        supera::ActionResult::Normal(match self {
            Self::Sub(a, b) => a - b,
            Self::Stop => return supera::ActionResult::Stop,
        })
    }
}

impl supera::SimpleStop for FnFMathAction {
    fn make_stop_command() -> Self {
        FnFMathAction::Stop
    }
}

impl supera::Command for FnFMathAction {
    type Result = ();
    fn execute(self) -> supera::ActionResult<FnFMathAction> {
        std::thread::sleep(std::time::Duration::from_millis(50));
        match self {
            Self::Sub => supera::ActionResult::Normal(()),
            Self::Stop => supera::ActionResult::Stop,
        }
    }
}

mod queue {
    use super::*;

    /// # Panics
    /// The runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn single_values() -> Result<(), Box<dyn std::error::Error>> {
        const COUNT: usize = 500_000;
        let mut outs = Vec::with_capacity(COUNT);
        supera::queue_single::SingleQueueAPI::<MathAction>::scope(|q| {
            let ma = MathAction::Sub(2, 1);
            for _ in 0..COUNT {
                q.send(ma).expect("Must send");
            }
            for _ in 0..COUNT {
                outs.push(q.recv().expect("Must recv"));
            }
        })?;
        assert_eq!(outs, vec![1; COUNT]);
        Ok(())
    }

    /// # Panics
    /// Runner manager can panic on close.
    /// Each runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn pool_values() -> Result<(), Box<dyn std::error::Error>> {
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
        })?;
        assert_eq!(outs, vec![1; COUNT]);
        for r in rs {
            r?;
        }
        Ok(())
    }

    #[test]
    /// # Panics
    /// The runner can panic.
    /// Sending and receiving the messages can panic.
    fn single_manual_close() -> Result<(), Box<dyn std::error::Error>> {
        let rs = unsafe { supera::queue_single::SingleQueueAPI::new() };
        rs.send(MathAction::Sub(3, 2))?;
        rs.recv()?;
        rs.close()?;
        Ok(())
    }

    /// # Panics
    /// Runner manager can panic on close.
    /// Each runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn pool_manual_close() -> Result<(), Box<dyn std::error::Error>> {
        let rs = unsafe { supera::queue_pool::PoolQueueAPI::<MathAction, 3>::new() };
        rs.send(MathAction::Sub(3, 2))?;
        rs.recv()?;
        for r in rs.close()? {
            r?;
        }
        Ok(())
    }

    #[test]
    fn return_from_scope() -> Result<(), Box<dyn std::error::Error>> {
        const COUNT: i32 = 2_500;

        let (rx, out) = supera::queue_pool::PoolQueueAPI::<MathAction, 3>::scope_and(|q| {
            let ma = MathAction::Sub(2, 1);
            let mut out = 0;
            for _ in 0..COUNT {
                q.send(ma)?;
            }
            for _ in 0..COUNT {
                out += q.recv()?;
            }
            Ok::<i32, Box<dyn std::error::Error>>(out)
        });
        for r in rx? {
            r?;
        }
        assert_eq!(out?, COUNT);
        Ok(())
    }
}

mod oneshot {
    use super::*;
    /// # Panics
    /// The runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn single_values() -> Result<(), Box<dyn std::error::Error>> {
        const COUNT: usize = 5_000;
        supera::oneshot_single::OneShotAPI::scope(|q| {
            for _ in 0..COUNT {
                let ma = MathAction::Sub(2, 1);
                let mr = q.send(ma).unwrap();
                let r = mr.recv().unwrap();
                assert_eq!(r, 1);
            }
        })?;
        Ok(())
    }

    /// # Panics
    /// Runner manager can panic on close.
    /// Each runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn pool_values() -> Result<(), Box<dyn std::error::Error>> {
        const COUNT: usize = 50_000;
        let runners = supera::oneshot_pool::OneShotPoolAPI::<MathAction, 10>::scope(|q| {
            for _ in 0..COUNT {
                let ma = MathAction::Sub(2, 1);
                let mr = q.send(ma).expect("Must send");
                let r = mr.recv().expect("Must recv");
                assert_eq!(r, 1);
            }
        })?;
        for r in runners {
            r?;
        }
        Ok(())
    }

    /// # Panics
    /// The runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn single_manual_close() -> Result<(), Box<dyn std::error::Error>> {
        use supera::oneshot_single::OneShotAPI;
        const COUNT: usize = 2_500;
        let q = unsafe { OneShotAPI::new() };
        for _ in 0..COUNT {
            let ma = MathAction::Sub(2, 1);
            let mr = q.send(ma)?;
            let r = mr.recv()?;
            assert_eq!(r, 1);
        }
        q.close()?;
        Ok(())
    }

    /// # Panics
    /// Runner manager can panic on close.
    /// Each runner can panic.
    /// Sending and receiving the messages can panic.
    #[test]
    fn pool_manual_close() -> Result<(), Box<dyn std::error::Error>> {
        use supera::oneshot_pool::OneShotPoolAPI;
        const COUNT: usize = 2_500;
        let q = unsafe { OneShotPoolAPI::<MathAction, 3>::new() };
        for _ in 0..COUNT {
            let ma = MathAction::Sub(2, 1);
            let mr = q.send(ma)?;
            let r = mr.recv()?;
            assert_eq!(r, 1);
        }
        for r in q.close()? {
            r?;
        }
        Ok(())
    }
}

mod fnf {
    use super::*;
    #[test]
    fn pool() {
        use supera::fnf_pool::PoolFnFAPI;
        const THREAD_COUNT: usize = 1022;
        const COUNT: usize = THREAD_COUNT * 40;
        let _ = PoolFnFAPI::<_, THREAD_COUNT>::scope(|pool| {
            for _ in 0..COUNT {
                pool.send(FnFMathAction::Sub).unwrap();
            }
        })
        .unwrap();
    }

    #[test]
    fn single() {
        use supera::fnf_single::SingleFnFAPI;
        const COUNT: usize = 120;
        let _ = SingleFnFAPI::scope(|pool| {
            for _ in 0..COUNT {
                pool.send(FnFMathAction::Sub).unwrap();
            }
        })
        .unwrap();
    }
}
