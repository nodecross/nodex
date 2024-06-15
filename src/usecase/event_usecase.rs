use crate::{
    repository::event_repository::{EventStoreRepository, EventStoreRequest},
    services::studio::Studio,
};

pub struct EventUsecase {
    repository: Box<dyn EventStoreRepository>,
}

impl EventUsecase {
    pub fn new() -> Self {
        EventUsecase {
            repository: Box::new(Studio::new()),
        }
    }

    pub async fn save(&self, request: EventStoreRequest) {
        self.repository.save(request).await;
    }
}
