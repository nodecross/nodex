use crate::{
    repository::attribute_repository::{AttributeStoreRepository, AttributeStoreRequest},
    services::studio::Studio,
};

pub struct AttributeUsecase {
    repository: Box<dyn AttributeStoreRepository>,
}

impl AttributeUsecase {
    pub fn new() -> Self {
        AttributeUsecase {
            repository: Box::new(Studio::new()),
        }
    }

    pub async fn save(&self, request: AttributeStoreRequest) -> anyhow::Result<()> {
        match self.repository.save(request).await {
            Ok(_) => {
                log::info!("save attribute");
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
    use crate::{
        repository::attribute_repository::{AttributeStoreRepository, AttributeStoreRequest},
        usecase::attribute_usecase::AttributeUsecase,
    };

    pub struct MockAttributeStoreRepository {}

    #[async_trait::async_trait]
    impl AttributeStoreRepository for MockAttributeStoreRepository {
        async fn save(&self, _: AttributeStoreRequest) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_save() {
        let usecase = AttributeUsecase {
            repository: Box::new(MockAttributeStoreRepository {}),
        };
        let _ = usecase
            .save(AttributeStoreRequest {
                key_name: "test".to_string(),
                value: "test".to_string(),
            })
            .await
            .map_err(|e| panic!("{:?}", e));
    }
}
