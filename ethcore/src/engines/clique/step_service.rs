use std::sync::Weak;
use std::time::Duration;
use io::{IoContext, IoHandler, TimerToken};
use engines::Engine;
use parity_machine::Machine;
use std::fmt::Debug;

pub struct StepService<M: Machine> {
  timeout: Duration,
  engine: Weak<Engine<M>>
}

impl<M: Machine> StepService<M> {
    /// New step caller by timeouts.
    pub fn new(engine: Weak<Engine<M>>, timeout: Duration) -> Self {
        StepService {
            engine: engine,
            timeout: timeout,
        }
    }
}

fn set_timeout<S: Sync + Send + Clone + 'static + Debug> (io: &IoContext<S>, timeout: Duration) {
        io.register_timer((1 as usize).into(), timeout)
                    .unwrap_or_else(|e| warn!(target: "engine", "Failed to set consensus step timeout: {}.", e))
}

impl<S, M> IoHandler<S> for StepService<M>
    where S: Sync + Send + Clone + 'static + Debug, M: Machine {

    fn initialize(&self, io: &IoContext<S>) {
        trace!(target: "engine", "Setting the step timeout to {:?}.", self.timeout);
        set_timeout(io, self.timeout);
    }

    /// Call step after timeout.
    fn timeout(&self, _io: &IoContext<S>, timer: TimerToken) {
        if let Some(engine) = self.engine.upgrade() {
            engine.step();
        }
    }

    /// Set a new timer on message.
    fn message(&self, io: &IoContext<S>, next: &S) {
		warn!(target: "engine", "Cannot set step timer")
    }
}

