//! The machine plate: every system this crate adds is in [`CarnageSystems`], the settings resources are
//! reflected and saveable, and the public components are reflected as components.
//!
//! The app is the whole crate: `GorePlugin` adds `CarnagePlugin` and every other plugin the crate has
//! (the strips, the flaymap, the wetmap, the lacerations, the viscera and the flesh material), so the
//! sets of every one of them are checked in one place. The `vfx` half (`CarnageVfxPlugin`) needs
//! `bevy_hanabi`, which has no Bevy 0.20 release, so it is not in this app.
//!
//! System names are the full path only with Bevy's `debug` feature, which the dev-dependency turns on;
//! the test asserts it found crate systems so a build without the feature cannot pass vacuously.

#![cfg(feature = "flesh")]

use std::any::TypeId;

use bevy::ecs::schedule::Schedules;
use bevy::prelude::*;
use bevy_carnage::cross_section::CrossSectionSettings;
use bevy_carnage::flaymap::{FlayCanvas, FlaySettings};
use bevy_carnage::fracture_modes::ModeSettings;
use bevy_carnage::laceration::{Laceration, LacerationBed};
use bevy_carnage::preset::{
    Bruising, ClothHoles, CutFaceBleed, DermisCanvas, Dressed, Flying, Gore, GoreBreakable,
    GorePlugin, GoreScene, Gut, Organ, RunDirection, Smeared, Soaking, Spurts, Standing, Tread,
    Wounds,
};
use bevy_carnage::viscera::{Mesentery, Strand, ViscSettings};
use bevy_carnage::wetmap::{WetCanvas, WetSettings};
use bevy_carnage::{
    Bleeding, CarnageSettings, CarnageSystems, DetachedPart, FractureBores, FractureProxy,
    FractureRegion, FractureSettings, FractureSubject,
};

const CRATE: &str = "bevy_carnage::";

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<Shader>()
        .init_asset::<StandardMaterial>()
        .add_plugins(GorePlugin);
    app.update();
    // A schedule that has not run (`FixedUpdate` has not accumulated a step after one `update`) has no
    // systems to read until it is built, so build every schedule first.
    let labels: Vec<_> = app.world().resource::<Schedules>().iter().map(|(_, schedule)| schedule.label()).collect();
    for label in labels {
        app.world_mut().schedule_scope(label, |world, schedule| {
            if let Err(error) = schedule.initialize(world) {
                panic!("schedule {label:?} does not build: {error}");
            }
        });
    }
    app
}

#[test]
fn machine_plate_every_system_is_in_the_root_set() {
    let app = app();
    let sets = [CarnageSystems.intern()];
    let schedules = app.world().resource::<Schedules>();
    let mut seen = 0;
    let mut stray = Vec::new();
    for (_label, schedule) in schedules.iter() {
        let Ok(systems) = schedule.systems() else {
            continue;
        };
        let members: Vec<_> = sets
            .iter()
            .filter_map(|set| schedule.graph().systems_in_set(*set).ok())
            .collect();
        for (key, system) in systems {
            let name = system.name().to_string();
            if !name.starts_with(CRATE) {
                continue;
            }
            seen += 1;
            if !members.iter().any(|m| m.contains(&key)) {
                stray.push(name);
            }
        }
    }
    assert!(
        seen > 0,
        "no system named `{CRATE}*` found; is Bevy's `debug` feature on?"
    );
    assert!(
        stray.is_empty(),
        "systems outside CarnageSystems: {stray:?}"
    );
}

#[test]
fn machine_plate_settings_are_reflected_and_saveable() {
    let app = app();
    let registry = app.world().resource::<AppTypeRegistry>().read();
    for (name, id) in [
        ("FractureSettings", TypeId::of::<FractureSettings>()),
        ("CarnageSettings", TypeId::of::<CarnageSettings>()),
        ("CrossSectionSettings", TypeId::of::<CrossSectionSettings>()),
        ("FlaySettings", TypeId::of::<FlaySettings>()),
        ("ModeSettings", TypeId::of::<ModeSettings>()),
        ("ViscSettings", TypeId::of::<ViscSettings>()),
        ("WetSettings", TypeId::of::<WetSettings>()),
    ] {
        assert!(
            registry.get_type_data::<ReflectResource>(id).is_some(),
            "{name} has no ReflectResource"
        );
        assert!(
            registry.get_type_data::<ReflectDefault>(id).is_some(),
            "{name} has no ReflectDefault"
        );
        #[cfg(feature = "serde")]
        {
            assert!(
                registry.get_type_data::<ReflectSerialize>(id).is_some(),
                "{name} has no ReflectSerialize"
            );
            assert!(
                registry.get_type_data::<ReflectDeserialize>(id).is_some(),
                "{name} has no ReflectDeserialize"
            );
        }
    }
}

#[test]
fn machine_plate_components_are_reflected() {
    let app = app();
    let registry = app.world().resource::<AppTypeRegistry>().read();
    for (name, id) in [
        ("FractureSubject", TypeId::of::<FractureSubject>()),
        ("FractureProxy", TypeId::of::<FractureProxy>()),
        ("FractureBores", TypeId::of::<FractureBores>()),
        ("DetachedPart", TypeId::of::<DetachedPart>()),
        ("FractureRegion", TypeId::of::<FractureRegion>()),
        ("Bleeding", TypeId::of::<Bleeding>()),
        ("FlayCanvas", TypeId::of::<FlayCanvas>()),
        ("WetCanvas", TypeId::of::<WetCanvas>()),
        ("Laceration", TypeId::of::<Laceration>()),
        ("LacerationBed", TypeId::of::<LacerationBed>()),
        ("Strand", TypeId::of::<Strand>()),
        ("Mesentery", TypeId::of::<Mesentery>()),
        ("Gore", TypeId::of::<Gore>()),
        ("GoreScene", TypeId::of::<GoreScene>()),
        ("Dressed", TypeId::of::<Dressed>()),
        ("Wounds", TypeId::of::<Wounds>()),
        ("Spurts", TypeId::of::<Spurts>()),
        ("GoreBreakable", TypeId::of::<GoreBreakable>()),
        ("DermisCanvas", TypeId::of::<DermisCanvas>()),
        ("Bruising", TypeId::of::<Bruising>()),
        ("Soaking", TypeId::of::<Soaking>()),
        ("ClothHoles", TypeId::of::<ClothHoles>()),
        ("Flying", TypeId::of::<Flying>()),
        ("Standing", TypeId::of::<Standing>()),
        ("CutFaceBleed", TypeId::of::<CutFaceBleed>()),
        ("Gut", TypeId::of::<Gut>()),
        ("Organ", TypeId::of::<Organ>()),
        ("Tread", TypeId::of::<Tread>()),
        ("Smeared", TypeId::of::<Smeared>()),
        ("RunDirection", TypeId::of::<RunDirection>()),
    ] {
        assert!(
            registry.get_type_data::<ReflectComponent>(id).is_some(),
            "{name} has no ReflectComponent"
        );
    }
}
