use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Table users
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::UserId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::TotalSpaceUsed).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Users::LastAutoSyncAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_user_id")
                    .table(Users::Table)
                    .col(Users::UserId)
                    .to_owned(),
            )
            .await?;

        // Table files
        manager
            .create_table(
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
                    .col(ColumnDef::new(Files::Version).string().not_null())
                    .col(ColumnDef::new(Files::IsLatest).boolean().not_null().default(true))
                    .col(ColumnDef::new(Files::AddedAt).timestamp().not_null())
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

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_files_id")
                    .table(Files::Table)
                    .col(Files::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_files_file_key")
                    .table(Files::Table)
                    .col(Files::FileKey)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_files_file_name")
                    .table(Files::Table)
                    .col(Files::FileName)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_files_file_path")
                    .table(Files::Table)
                    .col(Files::FilePath)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_files_content_type")
                    .table(Files::Table)
                    .col(Files::ContentType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_files_added_at")
                    .table(Files::Table)
                    .col(Files::AddedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_files_user_id_is_latest")
                    .table(Files::Table)
                    .col(Files::UserId)
                    .col(Files::IsLatest)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE FUNCTION set_file_added_at()
                RETURNS TRIGGER AS '
                BEGIN
                    IF TG_OP = ''INSERT'' THEN
                        NEW = ROW((NEW).id, (NEW).file_key, (NEW).user_id, 
                                  (NEW).file_name, (NEW).file_path, 
                                  (NEW).content_type, (NEW).content_size, (NEW).version, (NEW).is_latest, 
                                   NOW()::TIMESTAMP);
                    END IF;
                    RETURN NEW;
                END;
                ' LANGUAGE plpgsql;
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER trigger_set_file_added_at
                BEFORE INSERT ON files
                FOR EACH ROW
                EXECUTE FUNCTION set_file_added_at();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE FUNCTION update_user_on_file_change()
                RETURNS TRIGGER AS '
                BEGIN
                    UPDATE users
                    SET updated_at = NOW()::TIMESTAMP
                    WHERE user_id = (NEW).user_id;
                    RETURN NEW;
                END;
                ' LANGUAGE plpgsql;
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER trigger_update_user_on_file_insert
                AFTER INSERT ON files
                FOR EACH ROW
                EXECUTE FUNCTION update_user_on_file_change();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE FUNCTION set_user_updated_at()
                RETURNS TRIGGER AS '
                BEGIN
                    IF TG_OP = ''INSERT'' THEN
                        NEW = ROW((NEW).user_id, (NEW).total_space_used, NOW()::TIMESTAMP, (NEW).last_auto_sync_at);
                    END IF;
                    RETURN NEW;
                END;
                ' LANGUAGE plpgsql;
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER trigger_set_user_updated_at
                BEFORE INSERT ON users
                FOR EACH ROW
                EXECUTE FUNCTION set_user_updated_at();
                "#,
            )
            .await?;

        // Create trigger to increment user's total_space_used on file insert
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE FUNCTION increment_user_space()
                RETURNS TRIGGER AS '
                BEGIN
                    UPDATE users
                    SET total_space_used = total_space_used + (NEW).content_size
                    WHERE user_id = (NEW).user_id;
                    RETURN NEW;
                END;
                ' LANGUAGE plpgsql;
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER trigger_increment_user_space
                AFTER INSERT ON files
                FOR EACH ROW
                EXECUTE FUNCTION increment_user_space();
                "#,
            )
            .await?;

        // Create trigger to decrement user's total_space_used on file delete
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE FUNCTION decrement_user_space()
                RETURNS TRIGGER AS '
                BEGIN
                    UPDATE users
                    SET total_space_used = total_space_used - (OLD).content_size
                    WHERE user_id = (OLD).user_id;
                    RETURN OLD;
                END;
                ' LANGUAGE plpgsql;
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER trigger_decrement_user_space
                AFTER DELETE ON files
                FOR EACH ROW
                EXECUTE FUNCTION decrement_user_space();
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP TRIGGER IF EXISTS trigger_set_file_added_at ON files;
                DROP FUNCTION IF EXISTS set_file_added_at;
                DROP TRIGGER IF EXISTS trigger_update_user_on_file_insert ON files;
                DROP FUNCTION IF EXISTS update_user_on_file_change;
                DROP TRIGGER IF EXISTS trigger_set_user_updated_at ON users;
                DROP FUNCTION IF EXISTS set_user_updated_at;
                DROP TRIGGER IF EXISTS trigger_increment_user_space ON files;
                DROP FUNCTION IF EXISTS increment_user_space;
                DROP TRIGGER IF EXISTS trigger_decrement_user_space ON files;
                DROP FUNCTION IF EXISTS decrement_user_space;
                "#,
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Files::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

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
    Version,
    IsLatest,
    AddedAt,
}