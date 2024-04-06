use crate::{entities::*, OrmDataloader};
use async_graphql::{dataloader::DataLoader, dynamic::*};
use sea_orm::DatabaseConnection;
use seaography::{Builder, BuilderContext};

lazy_static::lazy_static! { static ref CONTEXT : BuilderContext = BuilderContext :: default () ; }

pub fn schema(
    database: DatabaseConnection,
    orm_dataloader: DataLoader<OrmDataloader>,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    let mut builder = Builder::new(&CONTEXT, database.clone());
    // seaography::register_entities!(builder, [badge, class,]);
    // 展开宏之后是下面的，但是不需要mutation所以把mutation注释掉了
    builder.register_entity::<badge::Entity>(
        <badge::RelatedEntity as sea_orm::Iterable>::iter()
            .map(|rel| seaography::RelationBuilder::get_relation(&rel, builder.context))
            .collect(),
    );
    builder = builder.register_entity_dataloader_one_to_one(badge::Entity, tokio::spawn);
    builder = builder.register_entity_dataloader_one_to_many(badge::Entity, tokio::spawn);
    // builder.register_entity_mutations::<badge::Entity, badge::ActiveModel>();
    builder.register_entity::<class::Entity>(
        <class::RelatedEntity as sea_orm::Iterable>::iter()
            .map(|rel| seaography::RelationBuilder::get_relation(&rel, builder.context))
            .collect(),
    );
    builder = builder.register_entity_dataloader_one_to_one(class::Entity, tokio::spawn);
    builder = builder.register_entity_dataloader_one_to_many(class::Entity, tokio::spawn);
    // builder.register_entity_mutations::<class::Entity, class::ActiveModel>();

    let schema = builder.schema_builder();
    let schema = if let Some(depth) = depth {
        schema.limit_depth(depth)
    } else {
        schema
    };
    let schema = if let Some(complexity) = complexity {
        schema.limit_complexity(complexity)
    } else {
        schema
    };
    schema.data(database).data(orm_dataloader).finish()
}
