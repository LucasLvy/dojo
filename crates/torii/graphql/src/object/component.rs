use async_graphql::dynamic::{
    Field, FieldFuture, FieldValue, InputValue, SubscriptionField, SubscriptionFieldFuture, TypeRef,
};
use async_graphql::{Name, Value};
use indexmap::IndexMap;
use sqlx::{Pool, Sqlite};
use tokio_stream::StreamExt;
use torii_core::simple_broker::SimpleBroker;
use torii_core::types::Component;

use super::connection::connection_output;
use super::{ObjectTrait, TypeMapping, ValueMapping};
use crate::constants::DEFAULT_LIMIT;
use crate::query::{query_all, query_by_id, query_total_count, ID};
use crate::types::ScalarType;

pub struct ComponentObject {
    pub type_mapping: TypeMapping,
}

impl Default for ComponentObject {
    // Eventually used for component metadata
    fn default() -> Self {
        Self {
            type_mapping: IndexMap::from([
                (Name::new("id"), TypeRef::named(TypeRef::ID)),
                (Name::new("name"), TypeRef::named(TypeRef::STRING)),
                (Name::new("classHash"), TypeRef::named(ScalarType::Felt252.to_string())),
                (Name::new("transactionHash"), TypeRef::named(ScalarType::Felt252.to_string())),
                (Name::new("createdAt"), TypeRef::named(ScalarType::DateTime.to_string())),
            ]),
        }
    }
}
impl ComponentObject {
    pub fn value_mapping(component: Component) -> ValueMapping {
        IndexMap::from([
            (Name::new("id"), Value::from(component.id)),
            (Name::new("name"), Value::from(component.name)),
            (Name::new("classHash"), Value::from(component.class_hash)),
            (Name::new("transactionHash"), Value::from(component.transaction_hash)),
            (
                Name::new("createdAt"),
                Value::from(component.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            ),
        ])
    }
}

impl ObjectTrait for ComponentObject {
    fn name(&self) -> &str {
        "component"
    }

    fn type_name(&self) -> &str {
        "Component"
    }

    fn type_mapping(&self) -> &TypeMapping {
        &self.type_mapping
    }

    fn resolve_one(&self) -> Option<Field> {
        Some(
            Field::new(self.name(), TypeRef::named_nn(self.type_name()), |ctx| {
                FieldFuture::new(async move {
                    let mut conn = ctx.data::<Pool<Sqlite>>()?.acquire().await?;
                    let id = ctx.args.try_get("id")?.string()?.to_string();
                    let component = query_by_id(&mut conn, "components", ID::Str(id)).await?;
                    let result = ComponentObject::value_mapping(component);
                    Ok(Some(Value::Object(result)))
                })
            })
            .argument(InputValue::new("id", TypeRef::named_nn(TypeRef::ID))),
        )
    }

    fn resolve_many(&self) -> Option<Field> {
        Some(Field::new(
            "components",
            TypeRef::named(format!("{}Connection", self.type_name())),
            |ctx| {
                FieldFuture::new(async move {
                    let mut conn = ctx.data::<Pool<Sqlite>>()?.acquire().await?;
                    let total_count =
                        query_total_count(&mut conn, "components", &Vec::new()).await?;
                    let data: Vec<Component> =
                        query_all(&mut conn, "components", DEFAULT_LIMIT).await?;
                    let components: Vec<ValueMapping> =
                        data.into_iter().map(ComponentObject::value_mapping).collect();

                    Ok(Some(Value::Object(connection_output(components, total_count))))
                })
            },
        ))
    }

    fn subscriptions(&self) -> Option<Vec<SubscriptionField>> {
        let name = format!("{}Registered", self.name());
        Some(vec![SubscriptionField::new(name, TypeRef::named_nn(self.type_name()), |_| {
            {
                SubscriptionFieldFuture::new(async {
                    Result::Ok(SimpleBroker::<Component>::subscribe().map(
                        |component: Component| {
                            Result::Ok(FieldValue::owned_any(ComponentObject::value_mapping(
                                component,
                            )))
                        },
                    ))
                })
            }
        })])
    }
}
