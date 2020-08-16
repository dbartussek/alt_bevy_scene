pub mod scene_to_value;
pub mod value_to_scene;

use bevy::{prelude::*, type_registry::TypeRegistry};

fn main() {
    App::build()
        .add_default_plugins()
        .register_component::<ComponentA>()
        .register_component::<ComponentB>()
        .add_startup_system(save_scene_system.thread_local_system())
        .run();
}

fn save_scene_system(_world: &mut World, resources: &mut Resources) {
    let mut world = World::new();
    world.spawn((
        ComponentA { x: 1.0, y: 2.0 },
        ComponentB {
            value: "hello".to_string(),
            ..ComponentB::from_resources(resources)
        },
    ));
    world.spawn((ComponentA { x: 3.0, y: 4.0 },));
    world.spawn(PbrComponents {
        ..Default::default()
    });

    let type_registry = resources.get::<TypeRegistry>().unwrap();
    let scene =
        Scene::from_world(&world, &type_registry.component.read().unwrap());

    let value = scene_to_value::scene_to_value(
        &type_registry.property.read().unwrap(),
        &scene,
    );

    std::fs::write("scene.tyf", value.to_string_pretty()).unwrap();

    let round_trip = value_to_scene::value_to_scene(
        &type_registry.property.read().unwrap(),
        &value,
    )
    .unwrap();
    let _ = round_trip;
}

#[derive(Properties, Default)]
struct ComponentA {
    pub x: f32,
    pub y: f32,
}

#[derive(Properties)]
struct ComponentB {
    pub value: String,
    #[property(ignore)]
    pub time_since_startup: std::time::Duration,
}

impl FromResources for ComponentB {
    fn from_resources(resources: &Resources) -> Self {
        let time = resources.get::<Time>().unwrap();
        ComponentB {
            time_since_startup: time.time_since_startup(),
            value: "Default Value".to_string(),
        }
    }
}
