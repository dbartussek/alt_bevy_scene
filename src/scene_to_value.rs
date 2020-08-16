use bevy::{
    property::{
        DynamicProperties, Property, PropertyType, PropertyTypeRegistry,
    },
    scene::{Entity, Scene},
};
use typed_format::value::{
    types::{Identifier, TypeIdentifier},
    Value,
};

fn property_to_value(
    registry: &PropertyTypeRegistry,
    prop: &Box<dyn Property>,
) -> Value {
    match prop.as_properties() {
        Some(props) => {
            dynamic_properties_to_value(registry, &props.to_dynamic())
        },
        None => {
            println!("Falling back on serializable for {}", prop.type_name());

            let value = Value::new(prop.serializable(registry).borrow());

            println!("{}\n", value.to_string_pretty());

            value
        },
    }
}

fn dynamic_properties_to_value(
    registry: &PropertyTypeRegistry,
    props: &DynamicProperties,
) -> Value {
    let t = TypeIdentifier::parse(&props.type_name).unwrap();

    match props.property_type {
        PropertyType::Map => Value::Struct(
            t,
            props
                .prop_names
                .iter()
                .zip(props.props.iter())
                .map(|(name, prop)| {
                    (
                        Identifier::from((&name) as &str),
                        property_to_value(registry, &prop),
                    )
                })
                .collect(),
        ),
        PropertyType::Seq => Value::List(
            props
                .props
                .iter()
                .map(|prop| property_to_value(registry, &prop))
                .collect(),
        ),
        other => {
            unimplemented!("Unimplemented PropertyType {:?}\n{:#?}", other, t)
        },
    }
}

fn entity_to_value(
    registry: &PropertyTypeRegistry,
    Entity { entity, components }: &Entity,
) -> Value {
    Value::TupleStruct(
        "Entity".into(),
        vec![
            Value::Number(entity.to_string()),
            Value::List(
                components
                    .iter()
                    .map(|props| dynamic_properties_to_value(registry, props))
                    .collect(),
            ),
        ],
    )
}

pub fn scene_to_value(registry: &PropertyTypeRegistry, scene: &Scene) -> Value {
    Value::List(
        scene
            .entities
            .iter()
            .map(|entity| entity_to_value(registry, entity))
            .collect(),
    )
}
