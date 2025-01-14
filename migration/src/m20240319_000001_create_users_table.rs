use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_query::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create enum for user roles
        manager
            .create_type(
                Type::create()
                    .as_enum(UserRole::Table)
                    .values([
                        UserRole::TenantAdmin,
                        UserRole::Manager,
                        UserRole::User,
                        UserRole::ReadOnly,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create enum for notification types
        manager
            .create_type(
                Type::create()
                    .as_enum(NotificationType::Table)
                    .values([
                        NotificationType::System,
                        NotificationType::Security,
                        NotificationType::Updates,
                        NotificationType::Mentions,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Users::TenantId).uuid().not_null())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::FullName).string().not_null())
                    .col(
                        ColumnDef::new(Users::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Users::Role)
                            .custom(UserRole::Table)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Users::Settings).json().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::LastLoginAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_tenant")
                            .from(Users::Table, Users::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_users_tenant_email")
                    .table(Users::Table)
                    .col(Users::TenantId)
                    .col(Users::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_tenant_username")
                    .table(Users::Table)
                    .col(Users::TenantId)
                    .col(Users::Username)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(UserRole::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(NotificationType::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    TenantId,
    Email,
    Username,
    FullName,
    IsActive,
    Role,
    Settings,
    CreatedAt,
    UpdatedAt,
    LastLoginAt,
}

#[derive(Iden)]
pub enum Tenants {
    Table,
    Id,
}

#[derive(Iden)]
pub enum UserRole {
    Table,
    #[iden = "tenant_admin"]
    TenantAdmin,
    #[iden = "manager"]
    Manager,
    #[iden = "user"]
    User,
    #[iden = "read_only"]
    ReadOnly,
}

#[derive(Iden)]
pub enum NotificationType {
    Table,
    #[iden = "system"]
    System,
    #[iden = "security"]
    Security,
    #[iden = "updates"]
    Updates,
    #[iden = "mentions"]
    Mentions,
}
