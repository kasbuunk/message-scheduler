use std::error::Error;

use chrono::prelude::*;
use async_trait::async_trait;

use crate::contract::Repository;
use crate::model::*;

pub struct RepositoryInMemory {
    schedules: Vec<MessageSchedule>,
}

#[async_trait]
impl Repository for RepositoryInMemory {
    async fn store_schedule(&mut self, schedule: &MessageSchedule) -> Result<(), Box<dyn Error>> {
        self.schedules.push(schedule.clone());

        Ok(())
    }

    async fn poll_batch(
        &self,
        before: DateTime<Utc>,
        batch_size: u32,
    ) -> Result<Vec<MessageSchedule>, Box<dyn Error>> {
        Ok(self
            .schedules
            .clone()
            .into_iter()
            .filter(|schedule| match schedule.next {
                None => false,
                Some(next) => next <= before,
            })
            .take(batch_size as usize)
            .collect())
    }

    async fn save(&mut self, schedule: &MessageSchedule) -> Result<(), Box<dyn Error>> {
        for stored_schedule in self.schedules.iter_mut() {
            if stored_schedule.id == schedule.id {
                *stored_schedule = schedule.clone();
            }
        }

        Ok(())
    }

    // reschedule is unnecessary for an in-memory implementation.
    async fn reschedule(&mut self, _schedule_id: &uuid::Uuid) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::time::{pause};

    #[tokio::test]
    async fn test_store() {
        let mut repository = RepositoryInMemory { schedules: vec![] };

        let now = Utc::now();
        let past = now - chrono::Duration::milliseconds(100);
        let future = now + chrono::Duration::milliseconds(100);

        let polled_schedules_empty = repository
            .poll_batch(Utc::now(), 100)
            .await
            .expect("poll batch should be ok");
        assert_eq!(polled_schedules_empty.len(), 0);

        let schedules = vec![
            MessageSchedule::new(
                SchedulePattern::Delayed(Delayed::new(past)),
                Message::NatsEvent(NatsEvent::new(
                    "ARBITRARY.subject".into(),
                    "arbitrary payload".into(),
                )),
            ),
            MessageSchedule::new(
                SchedulePattern::Delayed(Delayed::new(future)),
                Message::NatsEvent(NatsEvent::new(
                    "ARBITRARY.subject".into(),
                    "arbitrary payload".into(),
                )),
            ),
        ];
        let expected_polled_schedules: Vec<MessageSchedule> = vec![MessageSchedule {
            id: schedules[0].id.clone(),
            schedule_pattern: SchedulePattern::Delayed(Delayed::new(past)),
            message: schedules[0].message.clone(),
            next: Some(past),
            transmission_count: 0,
        }];

        for schedule in schedules.iter() {
            repository
                .store_schedule(schedule)
                .await
                .expect("store schedule should be ok");
        }

        let polled_schedules = repository
            .poll_batch(now, 100)
            .await
            .expect("poll batch should be ok");
        assert_eq!(polled_schedules, expected_polled_schedules);

        for schedule in schedules.iter() {
            let transmitted_message = schedule.transmitted();
            match transmitted_message {
                Ok(schedule) => match repository.save(&schedule).await {
                    Ok(()) => (),
                    Err(err) => panic!("failed to save: {err}"),
                },
                Err(err) => panic!("failed to transition to transmitted state: {err}"),
            };
        }

        pause();
        
        let polled_schedules_transmitted = repository
            .poll_batch(Utc::now(), 100)
            .await
            .expect("poll batch should be ok");
        assert_eq!(polled_schedules_transmitted, vec![]);
    }
}