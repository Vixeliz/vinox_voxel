use std::marker::PhantomData;

// use bevy::render::view::visibility;
use ndshape::{ConstShape, ConstShape3usize};
use serde::Serialize;

use crate::prelude::*;

// TODO: Replace block matches with a trait
#[derive(Default)]
pub struct GeoPalette {
    pub palette: Vec<BlockGeo>,
}

#[derive(Default)]
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
        geo_registry: Option<&GeometryRegistry>,
        vox_registry: Option<&R>,
    ) -> Option<usize>;

    fn to_match_idx(&self, match_pal: Option<&mut BlockMatches>) -> usize;
    /// These should return the uvs for the whole texture of this face this doesn't include the uvs for geometry faces
    fn to_texture_uv(
        &self,
        vox_registry: Option<&R>,
        asset_registry: Option<&AssetRegistry>,
    ) -> Option<[UVRect; 6]>;
    /// Returns if a side of this voxel will block that faces neighbor
    fn blocking_sides(
        &self,
        vox_registry: Option<&R>,
        geo_registry: Option<&GeometryRegistry>,
    ) -> Option<([bool; 6], Option<[bool; 6]>)>;

    fn light_level() -> Option<u8>;
    fn to_visibility(
        &self,
        vox_registry: Option<&R>,
        geo_registry: Option<&GeometryRegistry>,
    ) -> Option<VoxelVisibility>;
}

const BOUNDARY_EDGE: usize = CHUNK_SIZE + 2;
type BoundaryShape = ConstShape3usize<BOUNDARY_EDGE, BOUNDARY_EDGE, BOUNDARY_EDGE>;

pub struct ChunkBoundary<
    V: Voxel<R> + Clone + Serialize + Eq + Default,
    R: VoxRegistry<V> + Clone + Default,
> {
    pub geometry_pal: GeoPalette,
    voxels: Box<[RenderedBlockData; BoundaryShape::SIZE]>,
    phantom: PhantomData<V>,
    phantom_r: PhantomData<R>,
}

#[allow(dead_code)]
impl<
        V: Voxel<R> + Clone + Serialize + Eq + Default + RenderedVoxel<V, R>,
        R: VoxRegistry<V> + Clone + Default,
    > ChunkBoundary<V, R>
{
    pub fn new(
        center: ChunkData<V, R>,
        neighbors: Box<[ChunkData<V, R>; 26]>,
        voxel_registry: &R,
        geo_table: &GeometryRegistry,
        asset_registry: &AssetRegistry,
        // loadable_assets: &LoadableAssets,
        // texture_atlas: &TextureAtlas,
    ) -> Self {
        const MAX: usize = CHUNK_SIZE;
        const BOUND: usize = MAX + 1;
        let mut geo_pal = GeoPalette::default();
        let mut matching_voxels = BlockMatches::default();
        let voxels: Box<[RenderedBlockData; BoundaryShape::SIZE]> = (0..BoundaryShape::SIZE)
            .map(|idx| {
                let [x, y, z] = BoundaryShape::delinearize(idx);
                match (x, y, z) {
                    (0, 0, 0) => get_rend(
                        &neighbors[0],
                        MAX - 1,
                        MAX - 1,
                        MAX - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, 0, 1..=MAX) => get_rend(
                        &neighbors[1],
                        MAX - 1,
                        MAX - 1,
                        z - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, 0, BOUND) => get_rend(
                        &neighbors[2],
                        MAX - 1,
                        MAX - 1,
                        0,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, 1..=MAX, 0) => get_rend(
                        &neighbors[3],
                        MAX - 1,
                        y - 1,
                        MAX - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, 1..=MAX, 1..=MAX) => get_rend(
                        &neighbors[4],
                        MAX - 1,
                        y - 1,
                        z - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, 1..=MAX, BOUND) => get_rend(
                        &neighbors[5],
                        MAX - 1,
                        y - 1,
                        0,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, BOUND, 0) => get_rend(
                        &neighbors[6],
                        MAX - 1,
                        0,
                        MAX - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, BOUND, 1..=MAX) => get_rend(
                        &neighbors[7],
                        MAX - 1,
                        0,
                        z - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (0, BOUND, BOUND) => get_rend(
                        &neighbors[8],
                        MAX - 1,
                        0,
                        0,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 0, 0) => get_rend(
                        &neighbors[9],
                        x - 1,
                        MAX - 1,
                        MAX - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 0, 1..=MAX) => get_rend(
                        &neighbors[10],
                        x - 1,
                        MAX - 1,
                        z - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 0, BOUND) => get_rend(
                        &neighbors[11],
                        x - 1,
                        MAX - 1,
                        0,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 1..=MAX, 0) => get_rend(
                        &neighbors[12],
                        x - 1,
                        y - 1,
                        MAX - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 1..=MAX, 1..=MAX) => get_rend(
                        &center,
                        x - 1,
                        y - 1,
                        z - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, 1..=MAX, BOUND) => get_rend(
                        &neighbors[13],
                        x - 1,
                        y - 1,
                        0,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, BOUND, 0) => get_rend(
                        &neighbors[14],
                        x - 1,
                        0,
                        MAX - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, BOUND, 1..=MAX) => get_rend(
                        &neighbors[15],
                        x - 1,
                        0,
                        z - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (1..=MAX, BOUND, BOUND) => get_rend(
                        &neighbors[16],
                        x - 1,
                        0,
                        0,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 0, 0) => get_rend(
                        &neighbors[17],
                        0,
                        MAX - 1,
                        MAX - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 0, 1..=MAX) => get_rend(
                        &neighbors[18],
                        0,
                        MAX - 1,
                        z - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 0, BOUND) => get_rend(
                        &neighbors[19],
                        0,
                        MAX - 1,
                        0,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 1..=MAX, 0) => get_rend(
                        &neighbors[20],
                        0,
                        y - 1,
                        MAX - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 1..=MAX, 1..=MAX) => get_rend(
                        &neighbors[21],
                        0,
                        y - 1,
                        z - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, 1..=MAX, BOUND) => get_rend(
                        &neighbors[22],
                        0,
                        y - 1,
                        0,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, BOUND, 0) => get_rend(
                        &neighbors[23],
                        0,
                        0,
                        MAX - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, BOUND, 1..=MAX) => get_rend(
                        &neighbors[24],
                        0,
                        0,
                        z - 1,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),
                    (BOUND, BOUND, BOUND) => get_rend(
                        &neighbors[25],
                        0,
                        0,
                        0,
                        geo_table,
                        voxel_registry,
                        asset_registry,
                        &mut geo_pal,
                        //texture_atlas,
                        &mut matching_voxels,
                    ),

                    (_, _, _) => RenderedBlockData::default(),
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            voxels,
            geometry_pal: geo_pal,
            phantom: PhantomData,
            phantom_r: PhantomData,
        }
    }

    pub fn voxels(&self) -> &[RenderedBlockData; BoundaryShape::USIZE] {
        &self.voxels
    }

    pub const fn edge() -> usize {
        BOUNDARY_EDGE
    }

    pub const fn size() -> usize {
        BoundaryShape::SIZE
    }

    pub fn linearize(x: usize, y: usize, z: usize) -> usize {
        BoundaryShape::linearize([x, y, z])
    }

    pub fn delinearize(idx: usize) -> (usize, usize, usize) {
        let res = BoundaryShape::delinearize(idx);
        (res[0], res[1], res[2])
    }

    pub fn x_offset() -> usize {
        ChunkBoundary::<V, R>::linearize(1, 0, 0) - ChunkBoundary::<V, R>::linearize(0, 0, 0)
    }

    pub fn y_offset() -> usize {
        ChunkBoundary::<V, R>::linearize(0, 1, 0) - ChunkBoundary::<V, R>::linearize(0, 0, 0)
    }

    pub fn z_offset() -> usize {
        ChunkBoundary::<V, R>::linearize(0, 0, 1) - ChunkBoundary::<V, R>::linearize(0, 0, 0)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn get_rend<
    V: Voxel<R> + Clone + Serialize + Eq + Default + RenderedVoxel<V, R>,
    R: VoxRegistry<V> + Clone + Default,
>(
    chunk: &ChunkData<V, R>,
    x: usize,
    y: usize,
    z: usize,
    geo_registry: &GeometryRegistry,
    vox_registry: &R,
    asset_registry: &AssetRegistry,
    geo_pal: &mut GeoPalette,
    // texture_atlas: &TextureAtlas,
    matching_blocks: &mut BlockMatches,
) -> RenderedBlockData {
    let (x, y, z) = (x as u32, y as u32, z as u32);
    let voxel = chunk.get(RelativeVoxelPos::new(x, y, z));
    let geo_index = voxel.to_geo_idx(Some(geo_pal), Some(geo_registry), Some(vox_registry));
    let match_index = voxel.to_match_idx(Some(matching_blocks));
    let visibility = voxel.to_visibility(Some(vox_registry), None);
    let blocks_tuple = voxel.blocking_sides(Some(vox_registry), Some(geo_registry));
    let textures = voxel.to_texture_uv(Some(vox_registry), Some(asset_registry));
    RenderedBlockData {
        geo_index,
        // direction: voxel.direction,
        // top: voxel.top,
        match_index,
        textures,
        visibility: visibility.unwrap_or_default(),
        // has_direction: block_data.has_direction.unwrap_or(false),
        // exclusive_direction: block_data.exclusive_direction.unwrap_or(false),
        blocks: blocks_tuple
            .unwrap_or(([true, true, true, true, true, true], None))
            .0,
        // ([true, true, true, true, true, true]),
        blocks_self: blocks_tuple
            .unwrap_or(([true, true, true, true, true, true], None))
            .1,
        light: None,
    }
}

#[cfg(feature = "render")]
#[derive(Debug, PartialEq, Clone, Copy)]
/// This is the data that is actually used for rendering. We store it seperatly for performance
pub struct RenderedBlockData {
    pub geo_index: Option<usize>,
    pub match_index: usize,
    pub visibility: VoxelVisibility,
    pub textures: Option<[UVRect; 6]>,
    pub blocks: [bool; 6],
    pub blocks_self: Option<[bool; 6]>,
    pub light: Option<u8>,
}

#[cfg(feature = "render")]
impl Default for RenderedBlockData {
    fn default() -> Self {
        RenderedBlockData {
            visibility: VoxelVisibility::Empty,
            blocks: [false, false, false, false, false, false],
            blocks_self: None,
            // tex_variance: [false, false, false, false, false, false],
            textures: None,
            geo_index: None,
            match_index: 0,
            light: None,
        }
    }
}
