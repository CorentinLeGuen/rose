use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
                DROP TRIGGER IF EXISTS trigger_increment_user_space ON files;
                DROP FUNCTION IF EXISTS increment_user_space;
                DROP TRIGGER IF EXISTS trigger_decrement_user_space ON files;
                DROP FUNCTION IF EXISTS decrement_user_space;
                "#,
            )
            .await?;

        Ok(())
    }
}