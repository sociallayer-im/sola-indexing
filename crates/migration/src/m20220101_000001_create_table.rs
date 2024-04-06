use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Class::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Class::Id)
                            .big_unsigned()
                            .unique_key()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Class::ControllerId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Class::Register).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Badge::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Badge::Id)
                            .big_unsigned()
                            .unique_key()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Badge::ClassId).big_unsigned().not_null())
                    .col(ColumnDef::new(Badge::Publisher).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Badge::Table, Badge::ClassId)
                            .to(Class::Table, Class::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Badge::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Class::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Badge {
    Table,
    Id,
    ClassId,
    Publisher,
}

#[derive(DeriveIden)]
enum Class {
    Table,
    Id,
    ControllerId,
    Register,
}
