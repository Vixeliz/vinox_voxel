use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::utils::default;
use bevy_flycam::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vinox_voxel::{mesh::mesh::full_mesh, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(PlayerPlugin);
    app.add_startup_system(setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
) {
    let mut registry = BlockRegistry::default();
    registry.0.insert(
        "vinox:test".to_string(),
        Block {
            identifier: "vinox:test".to_string(),
            textures: None,
            geometry: Some(BlockGeometry::Block),
            auto_geo: None,
            visibility: Some(VoxelVisibility::Opaque),
            has_item: None,
        },
    );
    registry.0.insert(
        "vinox:slab".to_string(),
        Block {
            identifier: "vinox:slab".to_string(),
            textures: None,
            geometry: Some(BlockGeometry::Slab),
            auto_geo: None,
            visibility: Some(VoxelVisibility::Opaque),
            has_item: None,
        },
    );

    let mut chunk = ChunkData::<BlockData, BlockRegistry>::default();
    for y in 0..2 {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if y == 1 {
                    if x == CHUNK_SIZE - 1 || z == CHUNK_SIZE - 1 || x == 0 || z == 0 {
                        chunk.set(
                            RelativeVoxelPos::new(x as u32, y + 1, z as u32),
                            BlockData::new("vinox".to_string(), "test".to_string()),
                        );
                        continue;
                    } else {
                        chunk.set(
                            RelativeVoxelPos::new(x as u32, y + 1, z as u32),
                            BlockData::new("vinox".to_string(), "slab".to_string()),
                        );
                        continue;
                    }
                }
                chunk.set(
                    RelativeVoxelPos::new(x as u32, y + 1, z as u32),
                    BlockData::new("vinox".to_string(), "test".to_string()),
                );
            }
        }
    }

    let mut geo_table = GeometryRegistry(HashMap::default());
    geo_table.insert("vinox:block".to_string(), Geometry::default());
    geo_table.insert(
        "vinox:slab".to_string(),
        Geometry {
            namespace: "vinox".to_string(),
            name: "slab".to_string(),
            blocks: [false, false, true, false, false, false],
            element: BlockGeo {
                pivot: (0, 0, 0),
                rotation: (0, 0, 0),
                cubes: vec![FaceDescript {
                    uv: [
                        ((0, 0), (16, 8)),
                        ((0, 0), (16, 8)),
                        ((16, 16), (-16, -16)),
                        ((16, 16), (-16, -16)),
                        ((0, 0), (16, 8)),
                        ((0, 0), (16, 8)),
                    ],
                    discard: [false, false, false, false, false, false],
                    texture_variance: [false, false, false, false, false, false],
                    cull: [false, false, true, false, false, false],
                    origin: (0, 0, 0),
                    end: (16, 8, 16),
                    rotation: (0, 0, 0),
                    pivot: (8, 8, 8),
                }],
            },
        },
    );

    let mut asset_registry = AssetRegistry::from_block_textures(textures, HashMap::new());

    let mesh = full_mesh(
        &ChunkBoundary::<BlockData, BlockRegistry>::new(
            chunk,
            Box::default(),
            &registry,
            &geo_table,
            &asset_registry,
        ),
        &asset_registry.texture_atlas,
        IVec3::new(0, 0, 0),
    );
    // println!("{}", mesh.chunk_mesh.vertices.len());
    let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    bevy_mesh.set_indices(Some(Indices::U32(mesh.chunk_mesh.indices)));
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh.chunk_mesh.vertices.clone());
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.chunk_mesh.normals);
    // bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    if let Some(mesh_colors) = mesh.chunk_mesh.colors {
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, mesh_colors);
    }
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight::default(),
        transform: Transform::from_xyz(8.0, 3.0, 8.0),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(bevy_mesh),
        material: materials.add(StandardMaterial::from(Color::WHITE)),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}
