use super::dtos::SearchUserResponse;
use core::entities::users;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

pub struct SearchUserUseCase;

impl SearchUserUseCase {
    pub async fn execute(
        db: &DatabaseConnection,
        query: String, // Can be username or hashed phone
    ) -> Result<Option<SearchUserResponse>, String> {
        // Try searching by Username first
        let user = users::Entity::find()
            .filter(users::Column::Username.eq(query.clone()))
            .one(db)
            .await
            .map_err(|e| e.to_string())?;

        // If not found, try searching by Phone Number Hash (assuming query is passed as hex string or similar?
        // Actually, if we want to search by phone, the client might send the HASHED phone.
        // Let's assume the client sends the HASH in hex format if searching by phone.
        // But 'query' is String. We need to handle binary comparison if we want to search by hash.
        // For now, let's assume strict search by 'Username' or 'Phone Hash'
        // If the query looks like a hash (e.g. base64 or hex), we might try it.
        // However, schema says phone_number_hash is `binary`.
        
        // Let's implement strictly ID search for now as the core requirement. 
        // Phone search usually requires dedicated handling (client provides phone -> client hashes -> sends hash -> server compares).
        
        // If query length is high?
        
        // TODO: Handle phone hash search. For now, just Username.
        
        if user.is_none() {
             return Ok(None);
        }

        let u = user.unwrap();
        Ok(Some(SearchUserResponse {
            user_id: u.user_id,
            username: u.username,
            display_name: u.display_name,
            profile_picture: u.profile_picture,
        }))
    }
}
