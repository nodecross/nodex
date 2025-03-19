use protocol::cbor::types::Event;

use crate::{repository::event_repository::EventStoreRepository, services::studio::Studio};

pub struct EventUsecase<R>
where
    R: EventStoreRepository,
{
    repository: R,
}

impl EventUsecase<Studio> {
    pub fn new() -> Self {
        EventUsecase {
            repository: Studio::new(),
        }
    }
}

impl<R: EventStoreRepository> EventUsecase<R> {
    pub async fn save(&self, request: Vec<Event>) -> anyhow::Result<()> {
        match self.repository.save(request).await {
            Ok(_) => {
                log::info!("save event");
                Ok(())
            }
            Err(e) => {
                log::error!("{:?}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use protocol::cbor::types::{Event, TimeValue};

    use crate::{
        repository::event_repository::EventStoreRepository, usecase::event_usecase::EventUsecase,
    };

    pub struct MockEventStoreRepository {}

    impl EventStoreRepository for MockEventStoreRepository {
        async fn save(&self, _: Vec<Event>) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_save() {
        let usecase = EventUsecase {
            repository: MockEventStoreRepository {},
        };
        let _ = usecase
            .save(vec![Event {
                key: "test".to_string(),
                details: vec![TimeValue(chrono::Utc::now(), "test".to_string())],
            }])
            .await
            .map_err(|e| panic!("{:?}", e));
    }
}
