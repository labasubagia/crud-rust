use std::sync::Arc;

use uuid::Uuid;

use crate::{
    config::Config,
    model::{
        error::{AppError, AppErrorCode},
        item::Item,
    },
    repository::Repository,
};

pub struct ItemService {
    repo: Arc<Repository>,
}

impl ItemService {
    pub fn new(_: Arc<Config>, repo: Arc<Repository>) -> Self {
        Self { repo }
    }

    pub async fn get(&self, id: String) -> Result<Item, AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item ID cannot be empty".to_string(),
            });
        }
        self.repo.item.get(id).await
    }

    pub async fn list(&self) -> Result<Vec<Item>, AppError> {
        self.repo.item.list().await
    }

    pub async fn create(&self, name: String) -> Result<Item, AppError> {
        let name = name.trim().to_lowercase();
        if name.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item name cannot be empty".to_string(),
            });
        }

        let new_item = Item {
            id: Uuid::new_v4().to_string(),
            name,
        };
        self.repo.item.add(new_item).await
    }

    pub async fn update(&self, id: String, name: String) -> Result<Item, AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item ID cannot be empty".to_string(),
            });
        }

        let name = name.trim().to_lowercase();
        if name.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item name cannot be empty".to_string(),
            });
        }

        self.repo.item.update(id, name).await
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item ID cannot be empty".to_string(),
            });
        }

        self.repo.item.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::item::MockItemRepository;

    use super::*;

    #[tokio::test]
    async fn test_create_item() {
        let mut mock_item_repo = MockItemRepository::new();

        mock_item_repo
            .expect_add()
            .withf(|item: &Item| item.name == "test item")
            .returning(|item| Box::pin(async move { Ok(item) }));

        let service = ItemService::new(
            Arc::new(Config::default()),
            Arc::new(Repository {
                item: Arc::new(mock_item_repo),
            }),
        );

        let item = service
            .create("Test Item".to_string())
            .await
            .expect("failed to create item");
        assert_eq!(item.name, "test item");
    }

    #[tokio::test]
    async fn test_get_item() {
        let mut mock_item_repo = MockItemRepository::new();
        let item = Item {
            id: "123".to_string(),
            name: "test item".to_string(),
        };
        mock_item_repo
            .expect_get()
            .withf(|id| id == "123")
            .returning(move |_| {
                Box::pin({
                    let value = item.clone();
                    async move { Ok(value) }
                })
            });

        let service = ItemService::new(
            Arc::new(Config::default()),
            Arc::new(Repository {
                item: Arc::new(mock_item_repo),
            }),
        );

        let fetched_item = service
            .get("123".to_string())
            .await
            .expect("failed to get item");
        assert_eq!(fetched_item.id, "123");
        assert_eq!(fetched_item.name, "test item");
    }

    #[tokio::test]
    async fn test_list_items() {
        let mut mock_item_repo = MockItemRepository::new();
        let items = vec![
            Item {
                id: "1".to_string(),
                name: "item one".to_string(),
            },
            Item {
                id: "2".to_string(),
                name: "item two".to_string(),
            },
        ];
        mock_item_repo.expect_list().returning(move || {
            Box::pin({
                let value = items.clone();
                async move { Ok(value.clone()) }
            })
        });

        let service = ItemService::new(
            Arc::new(Config::default()),
            Arc::new(Repository {
                item: Arc::new(mock_item_repo),
            }),
        );

        let fetched_items = service.list().await.expect("failed to list items");
        assert_eq!(fetched_items.len(), 2);
        assert_eq!(fetched_items[0].name, "item one");
        assert_eq!(fetched_items[1].name, "item two");
    }

    #[tokio::test]
    async fn test_update_item() {
        let mut mock_item_repo = MockItemRepository::new();
        let item = Item {
            id: "123".to_string(),
            name: "updated item".to_string(),
        };
        mock_item_repo
            .expect_update()
            .withf(|id, name| id == "123" && name == "updated item")
            .returning(move |_, _| {
                Box::pin({
                    let value = item.clone();
                    async move { Ok(value.clone()) }
                })
            });

        let service = ItemService::new(
            Arc::new(Config::default()),
            Arc::new(Repository {
                item: Arc::new(mock_item_repo),
            }),
        );

        let updated_item = service
            .update("123".to_string(), "Updated Item".to_string())
            .await
            .expect("failed to update item");
        assert_eq!(updated_item.id, "123");
        assert_eq!(updated_item.name, "updated item");
    }

    #[tokio::test]
    async fn test_delete_item() {
        let mut mock_item_repo = MockItemRepository::new();
        mock_item_repo
            .expect_delete()
            .withf(|id| id == "123")
            .returning(move |_| Box::pin(async move { Ok(()) }));

        let service = ItemService::new(
            Arc::new(Config::default()),
            Arc::new(Repository {
                item: Arc::new(mock_item_repo),
            }),
        );

        let result = service.delete("123".to_string()).await;
        assert!(result.is_ok());
    }
}
