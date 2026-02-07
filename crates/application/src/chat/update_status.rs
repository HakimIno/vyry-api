use vyry_core::entities::message_deliveries;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, ActiveModelTrait,
};
use chrono::Utc;
use crate::AppError;

pub struct UpdateDeliveryStatusUseCase;

impl UpdateDeliveryStatusUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        message_id: i64,
        device_id: i64,
        status: super::dtos::DeliveryStatusType,
    ) -> Result<(), AppError> {
        // Find the delivery record
        let delivery = message_deliveries::Entity::find()
            .filter(message_deliveries::Column::MessageId.eq(message_id))
            .filter(message_deliveries::Column::DeviceId.eq(device_id))
            .one(db)
            .await
            .map_err(AppError::from)?;

        if let Some(delivery) = delivery {
            let mut active_delivery: message_deliveries::ActiveModel = delivery.into();
            
            match status {
                super::dtos::DeliveryStatusType::Delivered => {
                    active_delivery.delivered_at = Set(Some(Utc::now().into()));
                },
                super::dtos::DeliveryStatusType::Read => {
                    active_delivery.read_at = Set(Some(Utc::now().into()));
                },
            }

            active_delivery.update(db).await.map_err(AppError::from)?;
        }

        Ok(())
    }
}
