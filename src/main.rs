use std::marker::PhantomData;

use bevy::{
    ecs::{
        component::{ComponentHooks, ComponentId, StorageType},
        system::SystemId,
        world::DeferredWorld,
    },
    prelude::*,
    utils::HashMap,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<AssetSystems>()
        .add_systems(Startup, startup)
        // NOTE: I'm adding the system to "PostStartup" to prevent the console from being spammed just to prove the design here.
        //       In practice, you will be adding this proxy system to "Update" or similar.
        .add_systems(PostStartup, proxy_system)
        .run();
}

#[derive(Asset, TypePath)]
struct CustomAssetTypeA;

#[derive(Asset, TypePath)]
struct CustomAssetTypeB;

#[derive(Resource, Default)]
struct AssetSystems(HashMap<ComponentId, SystemId>);

// I'm using phantom here, but in your case it will be an instance of A.
struct AssetComponent<A> {
    _phantom: PhantomData<A>,
}

impl<A> Default for AssetComponent<A> {
    fn default() -> Self {
        Self {
            _phantom: default(),
        }
    }
}

impl<A> Component for AssetComponent<A>
where
    A: Send + Sync + 'static,
{
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(Self::on_add);
    }
}

impl<A: 'static> AssetComponent<A> {
    fn on_add(mut world: DeferredWorld<'_>, _: Entity, component_id: ComponentId) {
        // check if we have already added a system for this asset type
        let asset_systems = world.resource::<AssetSystems>();
        if asset_systems.0.contains_key(&component_id) {
            println!(
                "System already added for {}, skipping!",
                std::any::type_name::<A>()
            );
            return;
        }

        // register the system and then add it to the hashmap
        let system_id = world.commands().register_system(generic_system::<A>);
        world
            .resource_mut::<AssetSystems>()
            .0
            .insert(component_id, system_id);

        println!("Added system for {}!", std::any::type_name::<A>())
    }
}

fn generic_system<A: 'static>() {
    // type_name will prove we have a running system for a given type
    println!("System for {:} running!", std::any::type_name::<A>());
}

fn proxy_system(asset_systems: Res<AssetSystems>, mut commands: Commands) {
    // the proxy system is the one that is registered in the schedule, it runs the registered systems
    for (_, system) in asset_systems.0.iter() {
        // run systems for each asset type
        commands.run_system(*system);
    }
}

fn startup(mut commands: Commands) {
    // spawn some components related to dummy asset types
    commands.spawn(AssetComponent::<CustomAssetTypeA>::default());
    commands.spawn(AssetComponent::<CustomAssetTypeB>::default());

    // spawn a component the same as one already spawned to prove duplicate systems aren't added by the hook
    commands.spawn(AssetComponent::<CustomAssetTypeA>::default());
}
