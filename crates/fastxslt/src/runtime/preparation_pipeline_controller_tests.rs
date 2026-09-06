//! Deterministic AR-0020 mechanics for a fixed-budget role-flexible dispatcher.

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchedulingMode {
    CombinedLocal,
    PreparationAhead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextWork {
    ExecuteReady,
    PrepareAhead,
    PrepareThenExecuteLocally,
    Idle,
}

#[derive(Debug, Clone, Copy)]
struct ControllerLimits {
    total_workers: usize,
    starvation_windows_to_activate: usize,
    relief_windows_to_deactivate: usize,
    minimum_active_windows: usize,
    ready_byte_stop: usize,
    oldest_ready_stop: Duration,
    small_request_p95_stop: Duration,
}

#[derive(Debug, Clone, Copy, Default)]
struct PressureWindow {
    raw_requests_waiting: usize,
    ready_packets: usize,
    ready_bytes: usize,
    workers_waiting_for_execution: usize,
    oldest_ready_age: Duration,
    small_request_p95: Duration,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ControllerObservation {
    activation_transitions: usize,
    deactivation_transitions: usize,
    safety_retreats: usize,
    windows_in_preparation_ahead: usize,
}

#[derive(Debug)]
struct PreparationAheadController {
    limits: ControllerLimits,
    mode: SchedulingMode,
    sustained_starvation: usize,
    sustained_relief: usize,
    active_windows: usize,
    observation: ControllerObservation,
}

impl PreparationAheadController {
    fn new(limits: ControllerLimits) -> Self {
        assert!(
            limits.total_workers > 0,
            "the host worker ceiling is nonzero"
        );
        assert!(
            limits.starvation_windows_to_activate > 0,
            "activation requires sustained evidence"
        );
        assert!(
            limits.relief_windows_to_deactivate > 0,
            "deactivation requires sustained relief"
        );
        Self {
            limits,
            mode: SchedulingMode::CombinedLocal,
            sustained_starvation: 0,
            sustained_relief: 0,
            active_windows: 0,
            observation: ControllerObservation::default(),
        }
    }

    fn observe(&mut self, pressure: PressureWindow) {
        let safety_veto = pressure.ready_bytes >= self.limits.ready_byte_stop
            || (pressure.ready_packets > 0
                && pressure.oldest_ready_age >= self.limits.oldest_ready_stop)
            || pressure.small_request_p95 >= self.limits.small_request_p95_stop;
        if safety_veto {
            if self.mode == SchedulingMode::PreparationAhead {
                self.mode = SchedulingMode::CombinedLocal;
                self.observation.deactivation_transitions += 1;
                self.observation.safety_retreats += 1;
            }
            self.sustained_starvation = 0;
            self.sustained_relief = 0;
            self.active_windows = 0;
            return;
        }

        match self.mode {
            SchedulingMode::CombinedLocal => {
                let starvation = pressure.raw_requests_waiting > 0
                    && pressure.ready_packets == 0
                    && pressure.workers_waiting_for_execution > 0;
                self.sustained_starvation = if starvation {
                    self.sustained_starvation.saturating_add(1)
                } else {
                    0
                };
                if self.sustained_starvation >= self.limits.starvation_windows_to_activate {
                    self.mode = SchedulingMode::PreparationAhead;
                    self.sustained_starvation = 0;
                    self.active_windows = 0;
                    self.observation.activation_transitions += 1;
                }
            }
            SchedulingMode::PreparationAhead => {
                self.active_windows = self.active_windows.saturating_add(1);
                self.observation.windows_in_preparation_ahead = self
                    .observation
                    .windows_in_preparation_ahead
                    .saturating_add(1);
                let relief = pressure.ready_packets > 0
                    || pressure.workers_waiting_for_execution == 0
                    || pressure.raw_requests_waiting == 0;
                self.sustained_relief = if relief {
                    self.sustained_relief.saturating_add(1)
                } else {
                    0
                };
                if self.active_windows >= self.limits.minimum_active_windows
                    && self.sustained_relief >= self.limits.relief_windows_to_deactivate
                {
                    self.mode = SchedulingMode::CombinedLocal;
                    self.sustained_relief = 0;
                    self.active_windows = 0;
                    self.observation.deactivation_transitions += 1;
                }
            }
        }
    }

    fn next_work(&self, pressure: PressureWindow) -> NextWork {
        if pressure.ready_packets > 0 {
            NextWork::ExecuteReady
        } else if pressure.raw_requests_waiting == 0 {
            NextWork::Idle
        } else if self.mode == SchedulingMode::PreparationAhead {
            NextWork::PrepareAhead
        } else {
            NextWork::PrepareThenExecuteLocally
        }
    }

    fn mode(&self) -> SchedulingMode {
        self.mode
    }

    fn total_worker_ceiling(&self) -> usize {
        self.limits.total_workers
    }

    fn observation(&self) -> ControllerObservation {
        self.observation
    }
}

fn limits() -> ControllerLimits {
    ControllerLimits {
        total_workers: 10,
        starvation_windows_to_activate: 3,
        relief_windows_to_deactivate: 2,
        minimum_active_windows: 3,
        ready_byte_stop: 1_000,
        oldest_ready_stop: Duration::from_millis(20),
        small_request_p95_stop: Duration::from_millis(5),
    }
}

fn starvation() -> PressureWindow {
    PressureWindow {
        raw_requests_waiting: 8,
        workers_waiting_for_execution: 2,
        ..PressureWindow::default()
    }
}

#[test]
fn transient_starvation_does_not_activate_preparation_ahead() {
    let mut controller = PreparationAheadController::new(limits());
    controller.observe(starvation());
    controller.observe(PressureWindow {
        raw_requests_waiting: 8,
        ..PressureWindow::default()
    });
    controller.observe(starvation());

    assert_eq!(controller.mode(), SchedulingMode::CombinedLocal);
    assert_eq!(
        controller.next_work(starvation()),
        NextWork::PrepareThenExecuteLocally
    );
    assert_eq!(controller.observation().activation_transitions, 0);
}

#[test]
fn sustained_starvation_activates_ahead_but_ready_execution_has_priority() {
    let mut controller = PreparationAheadController::new(limits());
    for _ in 0..3 {
        controller.observe(starvation());
    }

    assert_eq!(controller.mode(), SchedulingMode::PreparationAhead);
    assert_eq!(controller.next_work(starvation()), NextWork::PrepareAhead);
    assert_eq!(
        controller.next_work(PressureWindow {
            raw_requests_waiting: 8,
            ready_packets: 1,
            ready_bytes: 100,
            ..PressureWindow::default()
        }),
        NextWork::ExecuteReady
    );
    assert_eq!(controller.total_worker_ceiling(), 10);
    assert_eq!(controller.observation().activation_transitions, 1);
}

#[test]
fn minimum_dwell_and_relief_hysteresis_prevent_one_window_flapping() {
    let mut controller = PreparationAheadController::new(limits());
    for _ in 0..3 {
        controller.observe(starvation());
    }
    let relief = PressureWindow {
        raw_requests_waiting: 8,
        ready_packets: 1,
        ready_bytes: 100,
        ..PressureWindow::default()
    };

    controller.observe(relief);
    controller.observe(starvation());
    controller.observe(relief);
    assert_eq!(controller.mode(), SchedulingMode::PreparationAhead);
    controller.observe(relief);

    assert_eq!(controller.mode(), SchedulingMode::CombinedLocal);
    assert_eq!(controller.observation().activation_transitions, 1);
    assert_eq!(controller.observation().deactivation_transitions, 1);
    assert_eq!(controller.observation().safety_retreats, 0);
}

#[test]
fn memory_age_and_small_tail_vetoes_retreat_immediately() {
    for veto in [
        PressureWindow {
            raw_requests_waiting: 8,
            ready_packets: 1,
            ready_bytes: 1_000,
            ..PressureWindow::default()
        },
        PressureWindow {
            raw_requests_waiting: 8,
            ready_packets: 1,
            ready_bytes: 100,
            oldest_ready_age: Duration::from_millis(20),
            ..PressureWindow::default()
        },
        PressureWindow {
            raw_requests_waiting: 8,
            small_request_p95: Duration::from_millis(5),
            ..PressureWindow::default()
        },
    ] {
        let mut controller = PreparationAheadController::new(limits());
        for _ in 0..3 {
            controller.observe(starvation());
        }
        controller.observe(veto);

        assert_eq!(controller.mode(), SchedulingMode::CombinedLocal);
        assert_eq!(controller.observation().safety_retreats, 1);
        assert_ne!(controller.next_work(veto), NextWork::PrepareAhead);
    }
}

#[test]
fn ready_work_and_empty_work_never_request_preparation_ahead() {
    let mut controller = PreparationAheadController::new(limits());
    for _ in 0..3 {
        controller.observe(starvation());
    }

    assert_eq!(
        controller.next_work(PressureWindow {
            ready_packets: 2,
            ready_bytes: 200,
            ..PressureWindow::default()
        }),
        NextWork::ExecuteReady
    );
    assert_eq!(
        controller.next_work(PressureWindow::default()),
        NextWork::Idle
    );
}
