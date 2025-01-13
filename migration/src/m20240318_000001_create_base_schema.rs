use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create tenants table
        manager
            .create_table(
                Table::create()
                    .table(Tenant::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tenant::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Tenant::Name).string().not_null())
                    .col(ColumnDef::new(Tenant::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Tenant::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        // Add indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_tenant_name")
                    .table(Tenant::Table)
                    .col(Tenant::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tenant::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Tenant {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}
