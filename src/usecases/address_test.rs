#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bson::oid::ObjectId;
    use mockall::predicate::eq;

    use crate::{
        entities::address_entity::AddressEntity, models::address_model::InsertAddress,
        repository::address_repository::MockAddressRepository, timer_helper::TimerHelper,
        usecases::address_usecase::AddressUsecase,
    };

    #[tokio::test]
    async fn create_address_test() {
        let mut address_repository_mock = MockAddressRepository::new();
        let timer_helper = TimerHelper::Mock.creation();

        let req = InsertAddress {
            public_key: String::from("test_public_key"),
        };
        let expected_id = ObjectId::parse_str("000000000000000000000001").unwrap();

        address_repository_mock
            .expect_get_by_address()
            .with(eq(req.public_key.clone()))
            .times(1)
            .returning(|_| Box::pin(async { Err(String::new()) }));

        address_repository_mock
            .expect_insert()
            .with(eq(AddressEntity::new(
                req.public_key.clone(),
                Arc::clone(&timer_helper),
            )))
            .returning(move |_| Box::pin(async move { Ok(expected_id.clone()) }));

        let address_usecase =
            AddressUsecase::creation(Arc::new(address_repository_mock), timer_helper);

        let result = match address_usecase.create_new_address(req).await {
            Ok(r) => r,
            Err(_) => panic!("create new address error"),
        };

        assert_eq!(result, expected_id);
    }
}
