use serde::Serialize;

use crate::prelude::*;

// TODO: Replace block matches with a trait
pub struct GeoPalette {
    pub palette: Vec<BlockGeo>,
}

pub struct BlockMatches {
    pub matches: Vec<String>,
}

#[cfg(feature = "render")]
/// A rendered voxel trait which returns the needed data for rendering. For arrays of 6 representing the 6 sides of a cube. this is which directions the elements correspond to
/// 00: West
/// 01: East
/// 02: Down
/// 03: Up
/// 04: South
/// 05: North

pub trait RenderedVoxel<
    V: Voxel<R> + Clone + Serialize + Eq + Default,
    R: VoxRegistry<V> + Clone + Default,
>
{
    fn to_geo_idx(
        &self,
        geo_pal: Option<&mut GeoPalette>,
        geo_registry: Option<GeometryRegistry>,
        vox_regisstry: Option<R>,
    ) -> Option<usize>;

    fn to_match_idx(&self, match_pal: Option<&mut BlockMatches>) -> usize;
    /// These should return the uvs for the whole texture of this face this doesn't include the uvs for geometry faces
    fn to_texture_uvs(
        &self,
        vox_regisstry: Option<R>,
        geo_registry: Option<GeometryRegistry>,
    ) -> Option<[(f32, f32); 6]>;
    /// Returns if a side of this voxel will block that faces neighbor
    fn blocking_sides(
        &self,
        vox_regisstry: Option<R>,
        geo_registry: Option<GeometryRegistry>,
    ) -> Option<[bool; 6]>;

    fn light_level() -> Option<u8>;
}

/*
    let match_index = if matching_blocks.contains(&trimed_identifier) {
        matching_blocks
            .iter()
            .position(|r| r.clone().eq(&trimed_identifier))
            .unwrap()
    } else {
        matching_blocks.push(trimed_identifier.clone());
        matching_blocks
            .iter()
            .position(|r| r.clone().eq(&trimed_identifier))
            .unwrap()
    };

*/

// #[cfg(feature = "render")]
// #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
// /// This is the data that is actually used for rendering. We store it seperatly for performance
// pub struct RenderedBlockData {
//     pub geo_index: usize,
//     pub match_index: usize,
//     pub visibility: VoxelVisibility,
//     pub textures: [usize; 6],
//     pub tex_variance: [bool; 6],
//     pub blocks: [bool; 6],
//     pub light: u8,
// }

// #[cfg(feature = "render")]
// impl Default for RenderedBlockData {
//     fn default() -> Self {
//         RenderedBlockData {
//             visibility: VoxelVisibility::Empty,
//             blocks: [false, false, false, false, false, false],
//             tex_variance: [false, false, false, false, false, false],
//             textures: [0, 0, 0, 0, 0, 0],
//             geo_index: 0,
//             match_index: 0,
//             light: 0,
//         }
//     }
// }
