use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // TODO: เขียนสคริปต์ migration ที่แท้จริงสำหรับ schema ของโปรเจกต์นี้
        // ตอนนี้ให้ทำเป็น no-op ไปก่อน เพื่อให้ระบบ migration ทำงานได้
        let _ = manager;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // TODO: เขียนสคริปต์ rollback ที่แท้จริง
        let _ = manager;
        Ok(())
    }
}
