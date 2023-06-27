use std::marker::PhantomData;

use bitvec::prelude::*;

use ndshape::{ConstShape, ConstShape3usize};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_ARR: u32 = CHUNK_SIZE as u32 - 1;
pub const TOTAL_CHUNK_SIZE: usize = (CHUNK_SIZE) * (CHUNK_SIZE) * (CHUNK_SIZE);

type ChunkShape = ConstShape3usize<CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Container {
    pub items: Vec<String>, // Hashmap would be better and may do more into implementing hashmyself at some point but this approach works for now
    pub max_size: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Storage<
    V: Voxel<R> + Clone + Serialize + Eq + Default,
    R: VoxRegistry<V> + Clone + Default,
> {
    Single(SingleStorage<V, R>),
    Multi(MultiStorage<V, R>),
}

/// Compressed storage for volumes with a single voxel type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SingleStorage<
    V: Voxel<R> + Clone + Serialize + Eq + Default,
    R: VoxRegistry<V> + Clone + Default,
> {
    size: usize,
    voxel: V,
    phantom: PhantomData<R>,
}

/// Palette compressed storage for volumes with multiple voxel types
/// Based on https://voxel.wiki/wiki/palette-compression/
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiStorage<
    V: Voxel<R> + Clone + Serialize + Eq + Default,
    R: VoxRegistry<V> + Clone + Default,
> {
    /// Size of chunk storage, in voxels
    size: usize,
    data: BitBuffer,
    palette: Vec<PaletteEntry<V, R>>,
    /// Palette capacity given size of indices
    /// Not necessarily equal to palette vector capacity
    palette_capacity: usize,
    /// Bit length of indices into the palette
    indices_length: usize,
}

impl<V: Voxel<R> + Clone + Serialize + Eq + Default, R: VoxRegistry<V> + Clone + Default>
    MultiStorage<V, R>
{
    fn new(size: usize, initial_voxel: V) -> Self {
        // Indices_length of 2 since this is only used for multiple voxel types
        let indices_length = 2;
        let initial_capacity = 2_usize.pow(indices_length as u32);
        let mut palette = Vec::with_capacity(initial_capacity);
        palette.push(PaletteEntry {
            voxel_type: initial_voxel,
            ref_count: size,
            phantom: PhantomData,
        });

        Self {
            size,
            data: BitBuffer::new(size * indices_length),
            palette,
            palette_capacity: initial_capacity,
            indices_length,
        }
    }

    fn grow_palette(&mut self) {
        let mut indices: Vec<usize> = Vec::with_capacity(self.size);
        for i in 0..self.size {
            indices.push(self.data.get(i * self.indices_length, self.indices_length));
        }

        self.indices_length <<= 1;
        let new_capacity = 2usize.pow(self.indices_length as u32);
        self.palette.reserve(new_capacity - self.palette_capacity);
        self.palette_capacity = new_capacity;

        self.data = BitBuffer::new(self.size * self.indices_length);

        for (i, idx) in indices.into_iter().enumerate() {
            self.data
                .set(i * self.indices_length, self.indices_length, idx);
        }
    }
}

impl<V: Voxel<R> + Clone + Serialize + Eq + Default, R: VoxRegistry<V> + Clone + Default>
    Storage<V, R>
{
    pub fn new(size: usize) -> Self {
        Self::Single(SingleStorage {
            size,
            voxel: V::default(),
            phantom: PhantomData,
        })
    }

    fn toggle_storage_type(&mut self) {
        *self = match self {
            Storage::Single(storage) => {
                Storage::Multi(MultiStorage::new(storage.size, storage.voxel.clone()))
            }
            Storage::Multi(storage) => {
                assert!(storage.palette.len() == 1);
                Storage::Single(SingleStorage {
                    size: storage.size,
                    voxel: storage.palette[0].voxel_type.clone(),
                    phantom: PhantomData,
                })
            }
        };
    }

    pub fn set(&mut self, target_idx: usize, voxel: V) {
        match self {
            Storage::Single(storage) => {
                if storage.voxel != voxel {
                    self.toggle_storage_type();
                    self.set(target_idx, voxel);
                }
            }
            Storage::Multi(storage) => {
                let palette_target_idx: usize = storage
                    .data
                    .get(target_idx * storage.indices_length, storage.indices_length);
                if let Some(target) = storage.palette.get_mut(palette_target_idx) {
                    target.ref_count -= 1;
                }

                // Look for voxel palette entry
                let palette_entry_voxel =
                    storage.palette.iter().enumerate().find_map(|(idx, entry)| {
                        if entry.voxel_type == voxel {
                            Some(idx)
                        } else {
                            None
                        }
                    });

                // Voxel type already in palette
                if let Some(idx) = palette_entry_voxel {
                    storage.data.set(
                        target_idx * storage.indices_length,
                        storage.indices_length,
                        idx,
                    );
                    storage
                        .palette
                        .get_mut(idx)
                        .expect("Failed to get palette entry of target voxel")
                        .ref_count += 1;

                    return;
                }

                // Overwrite target palette entry
                if let Some(target) = storage.palette.get_mut(palette_target_idx) {
                    if target.ref_count == 0 {
                        target.voxel_type = voxel;
                        target.ref_count = 1;

                        return;
                    }
                }

                // Create new palette entry
                //bevy::prelude::info!("Creating new voxel entry for {:?}", voxel);
                let new_entry_idx = if let Some((i, entry)) = storage
                    .palette
                    .iter_mut()
                    .enumerate()
                    .find(|(_i, entry)| entry.ref_count == 0)
                {
                    // Recycle a ref_count 0 entry if any exists
                    entry.voxel_type = voxel;
                    entry.ref_count = 1;

                    i
                } else {
                    // Create a new entry from scratch
                    if storage.palette.len() == storage.palette_capacity {
                        storage.grow_palette();
                    }

                    storage.palette.push(PaletteEntry {
                        voxel_type: voxel,
                        ref_count: 1,
                        phantom: PhantomData,
                    });

                    storage.palette.len() - 1
                };
                storage.data.set(
                    target_idx * storage.indices_length,
                    storage.indices_length,
                    new_entry_idx,
                );
            }
        }
    }

    pub fn get(&self, idx: usize) -> V {
        match self {
            Storage::Single(storage) => storage.voxel.clone(),
            Storage::Multi(storage) => {
                let palette_idx: usize = storage
                    .data
                    .get(idx * storage.indices_length, storage.indices_length);

                storage
                    .palette
                    .get(palette_idx)
                    .expect("Failed to get palette entry in voxel get")
                    .voxel_type
                    .clone()
            }
        }
    }

    pub fn trim(&mut self) {
        match self {
            Storage::Single(_) => (),
            Storage::Multi(storage) => {
                if storage.palette.len() == 1 {
                    self.toggle_storage_type();
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PaletteEntry<V: Voxel<R> + Clone + Serialize, R: VoxRegistry<V> + Clone + Default> {
    voxel_type: V,
    ref_count: usize,
    phantom: PhantomData<R>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BitBuffer {
    bytes: BitVec<u8, Lsb0>,
}

impl BitBuffer {
    /// Create a new BitBuffer
    /// size is specified in bits, not bytes
    fn new(size: usize) -> Self {
        Self {
            bytes: BitVec::repeat(false, size),
        }
    }

    /// Set arbitraty bits in BitBuffer.
    /// idx, bit_length and bits are specified in bits, not bytes
    fn set(&mut self, idx: usize, bit_length: usize, bits: usize) {
        self.bytes[idx..idx + bit_length].store_le::<usize>(bits);
    }

    /// Get arbitraty bits in BitBuffer.
    /// idx, bit_length are specified in bits, not bytes
    fn get(&self, idx: usize, bit_length: usize) -> usize {
        self.bytes[idx..idx + bit_length].load_le::<usize>()
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct RawChunk<
    V: Voxel<R> + Clone + Serialize + Eq + Default,
    R: VoxRegistry<V> + Clone + Default,
> {
    voxels: Storage<V, R>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct ChunkData<
    V: Voxel<R> + Clone + Serialize + Eq + Default,
    R: VoxRegistry<V> + Clone + Default,
> {
    voxels: Storage<V, R>,
    change_count: u16,
    dirty: bool,
}

impl<V: Voxel<R> + Clone + Serialize + Eq + Default, R: VoxRegistry<V> + Clone + Default> Default
    for ChunkData<V, R>
{
    fn default() -> Self {
        Self {
            voxels: Storage::new(ChunkShape::USIZE),
            change_count: 0,
            dirty: true,
        }
    }
}

#[allow(dead_code)]
impl<V: Voxel<R> + Clone + Serialize + Eq + Default, R: VoxRegistry<V> + Clone + Default>
    ChunkData<V, R>
{
    pub fn get(&self, pos: RelativeVoxelPos) -> V {
        self.voxels.get(Self::linearize(pos))
    }

    pub fn get_identifier(&self, pos: RelativeVoxelPos) -> String {
        let voxel = self.voxels.get(Self::linearize(pos));
        voxel.identifier()
    }

    pub fn set(&mut self, pos: RelativeVoxelPos, voxel: V) {
        self.voxels.set(Self::linearize(pos), voxel);
        self.change_count += 1;
        self.set_dirty(true);

        if self.change_count > 500 {
            self.voxels.trim();
            self.change_count = 0;
        }
    }

    pub fn is_uniform(&self) -> bool {
        match self.voxels {
            Storage::Single(_) => true,
            Storage::Multi(_) => false,
        }
    }

    pub fn is_empty(&self, registry: Option<&R>) -> bool {
        self.is_uniform()
            && self
                .get(RelativeVoxelPos(glam::UVec3::new(0, 0, 0).into()))
                .is_empty(registry)
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn trim(&mut self) {
        self.voxels.trim();
    }

    pub const fn size() -> u32 {
        ChunkShape::USIZE as u32
    }

    pub const fn usize() -> usize {
        ChunkShape::USIZE
    }

    pub const fn edge() -> usize {
        CHUNK_SIZE
    }

    #[inline]
    pub fn linearize(pos: RelativeVoxelPos) -> usize {
        ChunkShape::linearize([pos.x as usize, pos.y as usize, pos.z as usize])
    }

    #[inline]
    pub fn delinearize(idx: usize) -> RelativeVoxelPos {
        let res = ChunkShape::delinearize(idx);
        RelativeVoxelPos::new(res[0] as u32, res[1] as u32, res[2] as u32)
    }

    pub fn from_raw(raw_chunk: RawChunk<V, R>) -> Self {
        Self {
            voxels: raw_chunk.voxels,
            change_count: 0,
            dirty: false,
        }
    }

    pub fn to_raw(&self) -> RawChunk<V, R> {
        RawChunk {
            voxels: self.voxels.clone(),
        }
    }
}
