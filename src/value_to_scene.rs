use anyhow::anyhow;
use bevy::{
    property::{
        property_serde::MapPropertyDeserializer, DynamicProperties, Property,
        PropertyTypeRegistration, PropertyTypeRegistry,
    },
    scene::{Entity, Scene},
};
use serde::de::DeserializeSeed;
use typed_format::value::{deserializer::ValueDeserializer, Value};

fn value_to_property_box(
    registry: &PropertyTypeRegistry,
    value: &Value,
    type_registration: &PropertyTypeRegistration,
) -> anyhow::Result<Box<dyn Property>> {
    Ok(type_registration.deserialize(ValueDeserializer { value }, registry)?)
}

fn value_to_dynamic_properties(
    registry: &PropertyTypeRegistry,
    value: &Value,
) -> anyhow::Result<DynamicProperties> {
    Ok(match value {
        Value::Map(_) => MapPropertyDeserializer::new(registry)
            .deserialize(ValueDeserializer { value })?,
        Value::Struct(identifier, items) => {
            let mut properties = DynamicProperties::map();
            properties.type_name = identifier.to_string();

            let type_registration =
                registry.get(&properties.type_name).ok_or_else(|| {
                    anyhow!("Type registration missing for {}", identifier)
                })?;

            for (key, value) in items {
                let value =
                    value_to_property_box(registry, value, type_registration)?;
                properties.set_box(&key.0, value);
            }

            properties
        },
        _ => {
            return Err(anyhow!(
                "Cannot convert Value to DynamicProperties: {:#?}",
                value
            ))
        },
    })
}

fn value_to_entity(
    registry: &PropertyTypeRegistry,
    value: &Value,
) -> anyhow::Result<Entity> {
    match value {
        Value::Tuple(items) => {
            if let Some((entity, components)) =
                items.get(0).and_then(|entity| {
                    items
                        .get(1)
                        .map(|components| match components {
                            Value::List(components) => Some(components),
                            _ => None,
                        })
                        .flatten()
                        .map(|components| (entity, components))
                })
            {
                let entity = entity.deserialize()?;
                let components = components
                    .iter()
                    .map(|component| {
                        value_to_dynamic_properties(registry, component)
                    })
                    .collect::<anyhow::Result<Vec<DynamicProperties>>>()?;

                return Ok(Entity { entity, components });
            }
        },
        _ => (),
    }

    Err(anyhow!("Expected entity, found {:#?}", value))
}

pub fn value_to_scene(
    registry: &PropertyTypeRegistry,
    value: &Value,
) -> anyhow::Result<Scene> {
    match value {
        Value::List(entities) => {
            let entities = entities
                .iter()
                .map(|entity| value_to_entity(registry, entity))
                .collect::<anyhow::Result<Vec<Entity>>>()?;
            Ok(Scene { entities })
        },
        _ => Err(anyhow!("Expected entity list, found {:#?}", value)),
    }
}
