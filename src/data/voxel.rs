use crate::{mesh::chunk::RenderedVoxel, prelude::*};
use ahash::HashMap;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use strum::EnumString;

pub trait VoxRegistry<V: Voxel<Self> + Sized>
where
    Self: Sized,
{
    fn is_empty(&self, vox: V) -> bool;
}

pub trait Voxel<R: VoxRegistry<Self> + Sized>
where
    Self: Sized,
{
    /// Implement this to tell if a voxel is empty
    fn is_empty(&self, registry: Option<&R>) -> bool;
    /// Impelement this for true emptiness. For example you may want light to propogate through blocks that are partially empty so thats what is_empty returns true_empty should be blocks that don't have custom geometry and are not opaque.
    fn is_true_empty(&self, registry: Option<&R>) -> bool;
    /// Oposite of is_empty
    fn is_opaque(&self, registry: Option<&R>) -> bool;
    /// Identifier must be something that implements eq
    fn identifier(&self) -> String;
}

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone, Copy, Hash)]
pub enum VoxelVisibility {
    #[default]
    Empty,
    Opaque,
    Transparent,
}

// Anything optional here that is necessary for the game to function but we have a default value for ie texture or geometry
// NOTE: We will also take in any children blocks this block may have. ie any slab, fence, stair variant etc
/// 00: West
/// 01: East
/// 02: Down
/// 03: Up
/// 04: South
/// 05: North
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct Block {
    pub identifier: String, // TODO: Make sure that we only allow one namespace:name pair
    pub textures: Option<[Option<String>; 6]>,
    pub geometry: Option<BlockGeometry>,
    pub auto_geo: Option<Vec<BlockGeometry>>, // Contains strings of geometry we wan't to auto generate
    pub visibility: Option<VoxelVisibility>,
    pub has_item: Option<bool>, // Basically whether or not we should auto generate an item for this block                                // pub properties: Option<Vec<Box<BlockData>>>,
}

#[derive(Deref, DerefMut, Default, Clone, Serialize, Deserialize)]
pub struct BlockRegistry(pub HashMap<String, Block>);

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub struct UVRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[cfg(feature = "render")]
#[derive(Clone, Deserialize, Serialize)]
pub struct AssetRegistry {
    pub texture_uvs: HashMap<String, [UVRect; 6]>,
    pub texture_size: mint::Point2<f32>,
    // pub texture_atlas: TextureAtlas,
}

impl AssetRegistry {
    // pub fn from_block_textures(
    //     mut textures: ResMut<Assets<Image>>,
    //     block_textures: HashMap<String, [Handle<Image>; 6]>,
    // ) -> Self {
    //     let mut texture_atlas_builder = TextureAtlasBuilder::default();
    //     for handle in block_textures.values() {
    //         for item in handle {
    //             let Some(texture) = textures.get(item) else {
    //                 warn!("Failed to load texture");
    //     // warn!("{:?} did not resolve to an `Image` asset.", asset_server.get_handle_path(item));
    //     continue;
    //             };
    //             texture_atlas_builder.add_texture(item.clone(), texture);
    //         }
    //     }
    //     let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();
    //     let mut texture_indexes = HashMap::new();
    //     for identifier in block_textures.keys() {
    //         let mut texture_index = [0, 0, 0, 0, 0, 0];
    //         for (i, texture) in texture_index.iter_mut().enumerate() {
    //             *texture = texture_atlas
    //                 .get_texture_index(&block_textures.get(identifier).unwrap()[i])
    //                 .unwrap_or_default();
    //         }
    //         texture_indexes.insert(identifier.clone(), texture_index);
    //         println!("{texture_index:?}");
    //     }

    //     // let atlas_handle = texture_atlases.add(texture_atlas);
    //     AssetRegistry {
    //         texture_indexes,
    //         texture_atlas,
    //     }
    //     // AssetRegistry { texture_indexes: , texture_atlas:  }
    // }
}

impl VoxRegistry<BlockData> for BlockRegistry {
    fn is_empty(&self, vox: BlockData) -> bool {
        self.0
            .get(&vox.identifier)
            .is_some_and(|x| x.visibility.is_some_and(|y| y == VoxelVisibility::Empty))
    }
}

#[cfg(feature = "block")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Property {
    Bool,
    BoolArray,
    Int,
    IntArray,
    Float,
    FloatArray,
}

#[cfg(feature = "block")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct BlockData {
    pub identifier: String,
    pub last_tick: Option<u64>,
    pub properties: Option<Vec<(String, Property)>>,
}

#[cfg(feature = "block")]
impl RenderedVoxel<Self, BlockRegistry> for BlockData {
    fn to_geo_idx(
        &self,
        geo_pal: Option<&mut crate::mesh::chunk::GeoPalette>,
        geo_registry: Option<&GeometryRegistry>,
        vox_registry: Option<&BlockRegistry>,
    ) -> Option<usize> {
        if let Some(geo_registry) = geo_registry {
            if let Some(vox_registry) = vox_registry {
                if let Some(block_data) = vox_registry.get(&self.identifier) {
                    if let Some(geo_pal) = geo_pal {
                        let geo_data = geo_registry.get(
                            &block_data
                                .clone()
                                .geometry
                                .unwrap_or_default()
                                .get_geo_namespace(),
                        );

                        let geo_data_new = geo_data.unwrap().element.clone();
                        return Some(if geo_pal.palette.contains(&geo_data_new) {
                            geo_pal
                                .palette
                                .iter()
                                .position(|r| r.clone() == geo_data_new)
                                .unwrap()
                        } else {
                            geo_pal.palette.push(geo_data_new);
                            geo_pal.palette.len() - 1
                            // geo_pal
                            //     .palette
                            //     .iter()
                            //     .position(|r| r.clone() == geo_data_new)
                            //     .unwrap()
                        });
                    }
                }
            }
        }
        None
    }

    fn to_match_idx(&self, match_pal: Option<&mut crate::mesh::chunk::BlockMatches>) -> usize {
        if let Some(match_pal) = match_pal {
            let trimed_identifier = trim_geo_identifier(self.identifier.clone());

            if match_pal.matches.contains(&trimed_identifier) {
                match_pal
                    .matches
                    .iter()
                    .position(|r| r.clone().eq(&trimed_identifier))
                    .unwrap()
            } else {
                match_pal.matches.push(trimed_identifier.clone());
                match_pal
                    .matches
                    .iter()
                    .position(|r| r.clone().eq(&trimed_identifier))
                    .unwrap()
            }
        } else {
            0
        }
    }

    fn to_texture_uv(
        &self,
        _vox_registry: Option<&BlockRegistry>,
        asset_registry: Option<&AssetRegistry>,
    ) -> Option<[UVRect; 6]> {
        if let Some(asset_registry) = asset_registry {
            return asset_registry.texture_uvs.get(&self.identifier).copied();
        }
        None
    }

    fn blocking_sides(
        &self,
        vox_registry: Option<&BlockRegistry>,
        geo_registry: Option<&GeometryRegistry>,
    ) -> Option<([bool; 6], Option<[bool; 6]>)> {
        if let Some(geo_registry) = geo_registry {
            if let Some(vox_registry) = vox_registry {
                if let Some(block_data) = vox_registry.get(&self.identifier) {
                    if let Some(geo_data) = geo_registry.get(
                        &block_data
                            .clone()
                            .geometry
                            .unwrap_or_default()
                            .get_geo_namespace(),
                    ) {
                        return Some((geo_data.blocks, geo_data.blocks_self));
                    }
                }
            }
        }
        None
    }

    fn light_level() -> Option<u8> {
        None
    }

    fn to_visibility(
        &self,
        vox_registry: Option<&BlockRegistry>,
        _geo_registry: Option<&GeometryRegistry>,
    ) -> Option<VoxelVisibility> {
        if let Some(vox_registry) = vox_registry {
            if let Some(block) = vox_registry.get(&self.identifier) {
                return block.visibility;
            }
        }
        None
    }
}

#[cfg(feature = "block")]
impl Voxel<BlockRegistry> for BlockData {
    fn is_empty(&self, registry: Option<&BlockRegistry>) -> bool {
        !registry.is_some_and(|z| {
            z.get(&self.identifier)
                .is_some_and(|x| x.visibility.is_some_and(|y| y != VoxelVisibility::Empty))
        })
    }

    fn is_opaque(&self, registry: Option<&BlockRegistry>) -> bool {
        registry.is_some_and(|z| {
            z.get(&self.identifier)
                .is_some_and(|x| x.visibility.is_some_and(|y| y == VoxelVisibility::Opaque))
        })
    }

    fn is_true_empty(&self, registry: Option<&BlockRegistry>) -> bool {
        !registry.is_some_and(|z| {
            z.get(&self.identifier)
                .is_some_and(|x| x.visibility.is_some_and(|y| y != VoxelVisibility::Empty))
        }) && registry.is_some_and(|z| {
            z.get(&self.identifier).is_some_and(|x| {
                x.geometry.clone().unwrap_or_default().get_geo_namespace() == "vinox:block"
            })
        })
    }

    fn identifier(&self) -> String {
        self.identifier.clone()
    }
}

#[cfg(feature = "block")]
impl Default for BlockData {
    fn default() -> Self {
        BlockData {
            identifier: "vinox:air".to_string(),
            last_tick: None,
            properties: None,
        }
    }
}

#[cfg(feature = "block")]
impl BlockData {
    pub fn new(namespace: String, name: String) -> Self {
        BlockData {
            identifier: namespace + ":" + name.as_str(),
            ..Default::default()
        }
    }
}
