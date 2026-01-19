use sea_orm_migration::{async_trait, prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        
        // 1. tables
        manager.create_table(
            Table::create()
                .table(Users::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Users::UserId)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Users::TotalSpaceUsed)
                        .big_integer()
                        .not_null()
                        .default(0),
                )
                .col(
                    ColumnDef::new(Users::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Users::LastAutoSyncAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .to_owned(),
        )
        .await?;

        manager.create_table(
            Table::create()
                .table(Files::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Files::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(Files::FileKey).uuid().not_null())
                .col(ColumnDef::new(Files::UserId).uuid().not_null())
                .col(ColumnDef::new(Files::FileName).string().not_null())
                .col(ColumnDef::new(Files::FilePath).string().not_null())
                .col(ColumnDef::new(Files::ContentType).string().not_null())
                .col(ColumnDef::new(Files::ContentSize).big_integer().not_null())
                .col(ColumnDef::new(Files::S3VersionId).string().not_null())
                .col(
                    ColumnDef::new(Files::IsLatest)
                        .boolean()
                        .not_null()
                        .default(true),
                )
                .col(
                    ColumnDef::new(Files::AddedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_files_user_id")
                        .from(Files::Table, Files::UserId)
                        .to(Users::Table, Users::UserId)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

        // 2. indexes
        manager.create_index(
            Index::create()
                .if_not_exists()
                .name("idx_files_file_key")
                .table(Files::Table)
                .col(Files::FileKey)
                .to_owned(),
        )
        .await?;

        manager.create_index(
            Index::create()
                .if_not_exists()
                .name("idx_files_lookup_active")
                .table(Files::Table)
                .col(Files::UserId)
                .col(Files::FilePath)
                .col(Files::IsLatest)
                .to_owned(),
        )
        .await?;

        manager.create_index(
            Index::create()
                .if_not_exists()
                .name("idx_files_listing")
                .table(Files::Table)
                .col(Files::UserId)
                .col(Files::IsLatest)
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Files::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
    TotalSpaceUsed,
    UpdatedAt,
    LastAutoSyncAt,
}

#[derive(DeriveIden)]
enum Files {
    Table,
    Id,
    FileKey,
    UserId,
    FileName,
    FilePath,
    ContentType,
    ContentSize,
    S3VersionId,
    IsLatest,
    AddedAt,
}