use std::error::Error;
use std::thread;
use std::time;

use chrono::prelude::*;
use cron::Schedule;
use uuid::Uuid;

#[allow(unused_imports)]
use mockall::{automock, mock, predicate::*};

static BATCH_SIZE: u32 = 100;

#[derive(Clone)]
pub enum Repeat {
    Infinitely,
    Times(u32),
}

#[derive(Clone)]
pub struct Periodic {
    pub next: DateTime<Utc>,
    pub interval: chrono::Duration,
    pub repeat: Repeat,
}

#[derive(Clone)]
pub enum SchedulePattern {
    AtDateTime(DateTime<Utc>),
    Cron(Schedule, Repeat),
    PeriodicInterval(Periodic),
}

#[derive(Clone, PartialEq)]
pub enum State {
    // A process can be dedicated to transition a task's state from scheduled to queued.
    Scheduled,
    // A process is currently picking up this message to dispatch it.
    Doing,
    // The scheduled message has been transmitted.
    Done,
}

#[derive(Clone)]
pub enum Message {
    Event(String),
}

#[derive(Clone)]
pub struct MessageSchedule {
    id: Uuid,
    pub schedule_pattern: SchedulePattern,
    pub message: Message,
}

impl MessageSchedule {
    pub fn new(schedule_pattern: SchedulePattern, message: Message) -> MessageSchedule {
        MessageSchedule {
            id: Uuid::new_v4(),
            schedule_pattern,
            message,
        }
    }
}

#[cfg_attr(test, automock)]
pub trait Repository {
    fn store_schedule(&self, schedule: MessageSchedule) -> Result<(), Box<dyn Error>>;
    fn poll_batch(&self, batch_size: u32) -> Result<Vec<MessageSchedule>, Box<dyn Error>>;
    fn mark_done(&self, schedule_id: &Uuid) -> Result<(), Box<dyn Error>>;
    fn set_next_periodic(
        &self,
        schedule_id: &Uuid,
        next: &DateTime<Utc>,
        repetitions: &Repeat,
    ) -> Result<(), Box<dyn Error>>;
    fn reschedule(&self, schedule_id: &Uuid) -> Result<(), Box<dyn Error>>;
}

#[cfg_attr(test, automock)]
pub trait Transmitter {
    fn transmit(&self, message: Message) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum MetricEvent {
    Scheduled(bool),
    Polled(bool),
    Transmitted(bool),
    MarkedDone(bool),
    DecrementedPeriodic(bool),
    Rescheduled(bool),
}

#[cfg_attr(test, automock)]
pub trait Metrics {
    fn count(&self, event: MetricEvent);
}

pub struct MessageScheduler {
    // repository keeps the program stateless, by providing a storage interface to store and
    // retrieve message schedules.
    repository: Box<dyn Repository>,
    // transmitter sends the message according to the configured communication protocol.
    transmitter: Box<dyn Transmitter>,
    // now is used to inject a mockable function to simulate the system's current datetime.
    now: Box<dyn Fn() -> DateTime<Utc>>,
    // metrics measures events of interest.
    metrics: Box<dyn Metrics>,
}

impl MessageScheduler {
    pub fn new(
        repository: Box<dyn Repository>,
        transmitter: Box<dyn Transmitter>,
        now: Box<dyn Fn() -> DateTime<Utc>>,
        metrics: Box<dyn Metrics>,
    ) -> MessageScheduler {
        MessageScheduler {
            repository,
            transmitter,
            now,
            metrics,
        }
    }

    pub fn run(&self) -> ! {
        loop {
            match self.process_batch() {
                Ok(_) => (),
                Err(err) => println!("error: {:?}", err),
            };

            thread::sleep(time::Duration::from_millis(100));
        }
    }

    pub fn schedule(&self, when: SchedulePattern, what: Message) -> Result<(), Box<dyn Error>> {
        let schedule = MessageSchedule::new(when, what);
        match self.repository.store_schedule(schedule) {
            Ok(_) => {
                self.metrics.count(MetricEvent::Scheduled(true));
                Ok(())
            }
            Err(err) => {
                self.metrics.count(MetricEvent::Scheduled(false));
                Err(err)
            }
        }
    }

    // process_batch retrieves schedules that are overdue, and in scheduled state. It transitions them
    // to doing/queued, calls transmits them and then transitions it back to scheduled, done or
    // error. Or, errors should have a separate thing. We don't want any errors to meddle with
    // things that may errored as a one-off problem; likewise we need errors to be transparent by
    // metrics and logging.
    fn process_batch(&self) -> Result<(), Box<dyn Error>> {
        let schedules = match self.repository.poll_batch(BATCH_SIZE) {
            Ok(schedules) => {
                self.metrics.count(MetricEvent::Polled(true));
                schedules
            }
            Err(err) => {
                self.metrics.count(MetricEvent::Polled(false));
                return Err(err);
            }
        };

        schedules
            .iter()
            .filter(|schedule| match &schedule.schedule_pattern {
                SchedulePattern::AtDateTime(datetime) => datetime < &(self.now)(),
                SchedulePattern::PeriodicInterval(periodic) => periodic.next < (self.now)(),
                _ => panic!("Implement me"),
            })
            .map(|schedule| match self.transmit(schedule) {
                Ok(_) => {
                    self.metrics.count(MetricEvent::Transmitted(true));
                    Ok(())
                }
                Err(err) => {
                    self.metrics.count(MetricEvent::Transmitted(false));
                    Err(err)
                }
            })
            .filter(|result| result.is_err())
            .for_each(|err| println!("{:?}", err));

        Ok(())
    }

    fn transmit(&self, schedule: &MessageSchedule) -> Result<(), Box<dyn Error>> {
        let transmission_result = self.transmitter.transmit(schedule.message.clone());

        match transmission_result {
            Ok(_) => match &schedule.schedule_pattern {
                SchedulePattern::AtDateTime(_) => {
                    let state_transition_result = self.repository.mark_done(&schedule.id);
                    match state_transition_result {
                        Ok(_) => {
                            self.metrics.count(MetricEvent::MarkedDone(true));
                            Ok(())
                        }
                        Err(err) => {
                            self.metrics.count(MetricEvent::MarkedDone(false));
                            Err(err)
                        }
                    }
                }
                SchedulePattern::PeriodicInterval(period) => {
                    let next_datetime = period.next + period.interval;

                    match period.repeat {
                        Repeat::Infinitely => {
                            let repetitions = Repeat::Infinitely;
                            let state_transition_result = self.repository.set_next_periodic(
                                &schedule.id,
                                &next_datetime,
                                &repetitions,
                            );
                            match state_transition_result {
                                Ok(_) => {
                                    self.metrics.count(MetricEvent::DecrementedPeriodic(true));
                                    Ok(())
                                }
                                Err(err) => {
                                    self.metrics.count(MetricEvent::DecrementedPeriodic(false));
                                    Err(err)
                                }
                            }
                        }
                        Repeat::Times(0) => {
                            let state_transition_result = self.repository.mark_done(&schedule.id);
                            match state_transition_result {
                                Ok(_) => {
                                    self.metrics.count(MetricEvent::MarkedDone(true));
                                    Ok(())
                                }
                                Err(err) => {
                                    self.metrics.count(MetricEvent::MarkedDone(false));
                                    Err(err)
                                }
                            }
                        }
                        Repeat::Times(n) => {
                            let repetitions = Repeat::Times(n - 1);
                            self.repository.set_next_periodic(
                                &schedule.id,
                                &next_datetime,
                                &repetitions,
                            )
                        }
                    }
                }
                _ => panic!("Implement me"),
            },
            Err(transmission_err) => {
                match self.repository.reschedule(&schedule.id) {
                    Ok(_) => {
                        self.metrics.count(MetricEvent::Rescheduled(true));
                        Err(transmission_err)
                    }
                    Err(err) => {
                        self.metrics.count(MetricEvent::Rescheduled(false));
                        // This requires extra attention, since unrestored non-transmitted
                        // scheduled messages will remain stuck in a DOING state.
                        // It should be logged for manual intervention, retried or
                        // forcibly rescheduled.
                        Err(format!("{:?}: {:?}", transmission_err, err).into())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_schedule() {
        let mut repository = MockRepository::new();
        repository
            .expect_store_schedule()
            .returning(|_| Ok(()))
            .times(1);

        let transmitter = MockTransmitter::new();
        let mut metrics = MockMetrics::new();
        metrics
            .expect_count()
            .with(eq(MetricEvent::Scheduled(true)))
            .returning(|_| ())
            .times(1);

        let scheduler = MessageScheduler::new(
            Box::new(repository),
            Box::new(transmitter),
            Box::new(Utc::now),
            Box::new(metrics),
        );

        let now = Utc::now();
        let pattern = SchedulePattern::AtDateTime(now);

        let result = scheduler.schedule(pattern, Message::Event("DataBytes".into()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_schedule_fail() {
        let mut repository = MockRepository::new();
        repository
            .expect_store_schedule()
            .returning(|_| Err("Failed to schedule message".into()))
            .times(1);

        let transmitter = MockTransmitter::new();
        let mut metrics = MockMetrics::new();
        metrics
            .expect_count()
            .with(eq(MetricEvent::Scheduled(false)))
            .returning(|_| ())
            .times(1);

        let scheduler = MessageScheduler::new(
            Box::new(repository),
            Box::new(transmitter),
            Box::new(Utc::now),
            Box::new(metrics),
        );

        let now = Utc::now();
        let pattern = SchedulePattern::AtDateTime(now);

        let result = scheduler.schedule(pattern, Message::Event("DataBytes".into()));
        assert!(result.is_err());
    }

    fn new_schedule_delayed() -> MessageSchedule {
        MessageSchedule::new(
            SchedulePattern::AtDateTime(Utc::now() - chrono::Duration::milliseconds(10)),
            Message::Event("ArbitraryData".into()),
        )
    }

    fn new_schedule_periodic() -> MessageSchedule {
        MessageSchedule::new(
            SchedulePattern::PeriodicInterval(Periodic {
                next: Utc::now() - chrono::Duration::milliseconds(10),
                repeat: Repeat::Infinitely,
                interval: chrono::Duration::milliseconds(100),
            }),
            Message::Event("ArbitraryData".into()),
        )
    }

    type StateTransitionFn = dyn Fn(&Uuid) -> Result<(), Box<dyn Error + 'static>> + Send;
    type SetNextPeriodicFn =
        dyn Fn(&Uuid, &DateTime<Utc>, &Repeat) -> Result<(), Box<dyn Error + 'static>> + Send;

    enum ScheduleStateTransition {
        MarkDone(Box<StateTransitionFn>, bool),
        SetNextPeriodic(Box<SetNextPeriodicFn>, bool),
        Reschedule(Box<StateTransitionFn>, bool),
    }

    struct TransmissionTestCase {
        name: String,
        transmission: Box<dyn Fn(Message) -> Result<(), Box<dyn Error + 'static>> + Send>,
        schedule_state_transition: ScheduleStateTransition,
        success: bool,
    }

    #[test]
    fn test_transmission_and_appropriate_state_transition() {
        let test_cases = vec![
            TransmissionTestCase {
                name: "success".into(),
                transmission: Box::new(move |_| Ok(())),
                schedule_state_transition: ScheduleStateTransition::MarkDone(
                    Box::new(move |_| Ok(())),
                    true,
                ),
                success: true,
            },
            TransmissionTestCase {
                name: "fail_and_reschedule".into(),
                transmission: Box::new(move |_| Err("Let's hope this gets rescheduled.".into())),
                schedule_state_transition: ScheduleStateTransition::Reschedule(
                    Box::new(move |_| Ok(())),
                    true,
                ),
                success: false,
            },
            TransmissionTestCase {
                name: "transmit_but_fail_mark_done".into(),
                transmission: Box::new(move |_| Ok(())),
                schedule_state_transition: ScheduleStateTransition::MarkDone(
                    Box::new(move |_| Err("The schedule is stuck in doing now.".into())),
                    false,
                ),
                success: false,
            },
            TransmissionTestCase {
                name: "transmit_fail_and_reschedule_fail".into(),
                transmission: Box::new(move |_| Err("Even the reschedule hereafter fails".into())),
                schedule_state_transition: ScheduleStateTransition::Reschedule(
                    Box::new(move |_| Err("The schedule is stuck in doing now.".into())),
                    false,
                ),
                success: false,
            },
        ];

        for test_case in test_cases {
            let mut transmitter = MockTransmitter::new();
            let mut repository = MockRepository::new();
            let mut metrics = MockMetrics::new();

            transmitter
                .expect_transmit()
                .returning(test_case.transmission)
                .times(1);

            match test_case.schedule_state_transition {
                ScheduleStateTransition::MarkDone(callback, transition_succeeds) => {
                    repository.expect_mark_done().returning(callback).times(1);
                    metrics
                        .expect_count()
                        .with(eq(MetricEvent::MarkedDone(transition_succeeds)))
                        .returning(|_| ())
                        .times(1);
                }
                ScheduleStateTransition::Reschedule(callback, transition_succeeds) => {
                    repository.expect_reschedule().returning(callback).times(1);
                    metrics
                        .expect_count()
                        .with(eq(MetricEvent::Rescheduled(transition_succeeds)))
                        .returning(|_| ())
                        .times(1);
                }
                _ => panic!("Unexpected test case"),
            };

            let scheduler = MessageScheduler::new(
                Box::new(repository),
                Box::new(transmitter),
                Box::new(Utc::now),
                Box::new(metrics),
            );

            let schedule = new_schedule_delayed();

            let result = scheduler.transmit(&schedule);
            assert_eq!(
                result.is_ok(),
                test_case.success,
                "Test case failed: {}",
                test_case.name
            );
        }
    }

    #[test]
    fn test_periodic_interval_mock_errors() {
        let test_cases = vec![
            TransmissionTestCase {
                name: "success".into(),
                transmission: Box::new(move |_| Ok(())),
                schedule_state_transition: ScheduleStateTransition::SetNextPeriodic(
                    Box::new(move |_, _, _| Ok(())),
                    true,
                ),
                success: true,
            },
            TransmissionTestCase {
                name: "fail_and_reschedule".into(),
                transmission: Box::new(move |_| Err("Let's hope this gets rescheduled.".into())),
                schedule_state_transition: ScheduleStateTransition::Reschedule(
                    Box::new(move |_| Ok(())),
                    true,
                ),
                success: false,
            },
            TransmissionTestCase {
                name: "transmit_but_fail_state_transition".into(),
                transmission: Box::new(move |_| Ok(())),
                schedule_state_transition: ScheduleStateTransition::SetNextPeriodic(
                    Box::new(move |_, _, _| Err("The schedule is stuck in doing now.".into())),
                    false,
                ),
                success: false,
            },
            TransmissionTestCase {
                name: "transmit_fail_and_reschedule_fail".into(),
                transmission: Box::new(move |_| Err("Even the reschedule hereafter fails".into())),
                schedule_state_transition: ScheduleStateTransition::Reschedule(
                    Box::new(move |_| Err("The schedule is stuck in doing now.".into())),
                    false,
                ),
                success: false,
            },
        ];

        for test_case in test_cases {
            let mut transmitter = MockTransmitter::new();
            let mut repository = MockRepository::new();
            let mut metrics = MockMetrics::new();

            transmitter
                .expect_transmit()
                .returning(test_case.transmission)
                .times(1);

            match test_case.schedule_state_transition {
                ScheduleStateTransition::MarkDone(callback, transition_succeeds) => {
                    repository.expect_mark_done().returning(callback).times(1);
                    metrics
                        .expect_count()
                        .with(eq(MetricEvent::MarkedDone(transition_succeeds)))
                        .returning(|_| ())
                        .times(1);
                }
                ScheduleStateTransition::SetNextPeriodic(callback, transition_succeeds) => {
                    repository
                        .expect_set_next_periodic()
                        .returning(callback)
                        .times(1);
                    metrics
                        .expect_count()
                        .with(eq(MetricEvent::DecrementedPeriodic(transition_succeeds)))
                        .returning(|_| ())
                        .times(1);
                }
                ScheduleStateTransition::Reschedule(callback, transition_succeeds) => {
                    repository.expect_reschedule().returning(callback).times(1);
                    metrics
                        .expect_count()
                        .with(eq(MetricEvent::Rescheduled(transition_succeeds)))
                        .returning(|_| ())
                        .times(1);
                }
            };

            let scheduler = MessageScheduler::new(
                Box::new(repository),
                Box::new(transmitter),
                Box::new(Utc::now),
                Box::new(metrics),
            );

            let schedule = new_schedule_periodic();

            let result = scheduler.transmit(&schedule);
            assert_eq!(
                result.is_ok(),
                test_case.success,
                "Test case failed: {}",
                test_case.name
            );
        }
    }

    #[test]
    fn test_poll_datetimes_schedules_success() {
        let schedule_list = vec![new_schedule_delayed(), new_schedule_delayed()];
        let amount_schedules = schedule_list.len();
        let schedule_list_clone = schedule_list.clone();

        let mut repository = MockRepository::new();
        repository
            .expect_poll_batch()
            .times(1)
            .returning(move |batch_size| {
                assert_eq!(batch_size, BATCH_SIZE);
                Ok(schedule_list_clone.clone())
            });

        let mut publisher = MockTransmitter::new();
        publisher
            .expect_transmit()
            .times(amount_schedules)
            .returning(|_message| Ok(()));

        repository
            .expect_mark_done()
            .times(amount_schedules)
            .returning(move |schedule_id| {
                // This schedule belongs to the original list constructed.
                assert!(schedule_list
                    .clone()
                    .iter()
                    .any(|schedule| &schedule.id == schedule_id));

                Ok(())
            });

        let mut metrics = MockMetrics::new();
        metrics
            .expect_count()
            .with(eq(MetricEvent::Polled(true)))
            .returning(|_| ())
            .times(1);
        metrics
            .expect_count()
            .with(eq(MetricEvent::Transmitted(true)))
            .returning(|_| ())
            .times(amount_schedules);
        metrics
            .expect_count()
            .with(eq(MetricEvent::MarkedDone(true)))
            .returning(|_| ())
            .times(amount_schedules);

        let scheduler = MessageScheduler::new(
            Box::new(repository),
            Box::new(publisher),
            Box::new(Utc::now),
            Box::new(metrics),
        );

        let result = scheduler.process_batch();
        assert!(result.is_ok());
    }

    #[test]
    fn test_poll_transmit_fail() {
        let now = Utc::now();
        let message_data = "This is an arbitrary message";
        let ten_milliseconds = chrono::Duration::milliseconds(10);

        let schedules = vec![MessageSchedule::new(
            SchedulePattern::AtDateTime(now - ten_milliseconds),
            Message::Event(message_data.into()),
        )];

        let mut repository = MockRepository::new();
        repository
            .expect_poll_batch()
            .times(1)
            .returning(move |_batch_size| Ok(schedules.clone()));
        repository
            .expect_reschedule()
            .times(1)
            .returning(move |_id| Ok(()));

        let mut transmitter = MockTransmitter::new();
        transmitter
            .expect_transmit()
            .times(1)
            .returning(move |_message| Err("Message fails to transmit".into()));

        let mut metrics = MockMetrics::new();
        metrics
            .expect_count()
            .with(eq(MetricEvent::Polled(true)))
            .returning(|_| ())
            .times(1);
        metrics
            .expect_count()
            .with(eq(MetricEvent::Transmitted(false)))
            .returning(|_| ())
            .times(1);
        metrics
            .expect_count()
            .with(eq(MetricEvent::Rescheduled(true)))
            .returning(|_| ())
            .times(1);

        let scheduler = MessageScheduler::new(
            Box::new(repository),
            Box::new(transmitter),
            Box::new(Utc::now),
            Box::new(metrics),
        );

        let result = scheduler.process_batch();
        assert!(result.is_ok());
    }

    #[test]
    // The first message will succeed, the second fails to transmit.
    fn test_poll_datetimes_schedules_fail_transmit() {
        let now = Utc::now();
        let ten_milliseconds = chrono::Duration::milliseconds(10);

        let message_data_success = "This message will transmit";
        let message_data_failure = "This message will fail to transmit";
        let index_message_failure = 1;

        let schedule_list = vec![
            MessageSchedule::new(
                // Ready to process.
                SchedulePattern::AtDateTime(now - ten_milliseconds),
                Message::Event(message_data_success.into()),
            ),
            MessageSchedule::new(
                // Ready to process.
                SchedulePattern::AtDateTime(now - ten_milliseconds),
                Message::Event(message_data_failure.into()),
            ),
        ];
        let amount_schedules = schedule_list.len();
        let schedule_list_clone = schedule_list.clone();
        let schedule_list_clone_0 = schedule_list.clone();

        let mut repository = MockRepository::new();
        repository
            .expect_poll_batch()
            .times(1)
            .returning(move |batch_size| {
                assert_eq!(batch_size, BATCH_SIZE);
                Ok(schedule_list_clone.clone())
            });

        let mut transmitter = MockTransmitter::new();
        transmitter
            .expect_transmit()
            .times(amount_schedules)
            .returning(move |message| match message {
                Message::Event(data) if &data == message_data_success => Ok(()),
                Message::Event(data) if &data == message_data_failure => {
                    Err("Second message fails to transmit".into())
                }
                _ => panic!("Unexpected transmission"),
            });

        repository
            .expect_mark_done()
            .times(amount_schedules - 1)
            .returning(move |schedule_id| match schedule_id {
                id if id != &schedule_list[index_message_failure].id => Ok(()),
                _ => panic!("Unexpected message marked done"),
            });
        repository
            .expect_reschedule()
            .times(1)
            .returning(move |schedule_id| match schedule_id {
                id if id == &schedule_list_clone_0[index_message_failure].id => Ok(()),
                _ => panic!("Unexpected restoration of scheduled message"),
            });

        let mut metrics = MockMetrics::new();
        metrics
            .expect_count()
            .with(eq(MetricEvent::Polled(true)))
            .returning(|_| ())
            .times(1);

        // Metrics caused by success.
        metrics
            .expect_count()
            .with(eq(MetricEvent::Transmitted(true)))
            .returning(|_| ())
            .times(amount_schedules - 1);
        metrics
            .expect_count()
            .with(eq(MetricEvent::MarkedDone(true)))
            .returning(|_| ())
            .times(amount_schedules - 1);

        // Metrics caused by failure.
        metrics
            .expect_count()
            .with(eq(MetricEvent::Transmitted(false)))
            .returning(|_| ())
            .times(1);
        metrics
            .expect_count()
            .with(eq(MetricEvent::Rescheduled(true)))
            .returning(|_| ())
            .times(1);

        let scheduler = MessageScheduler::new(
            Box::new(repository),
            Box::new(transmitter),
            Box::new(Utc::now),
            Box::new(metrics),
        );

        let result = scheduler.process_batch();
        assert!(result.is_ok());
    }
}