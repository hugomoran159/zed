use crate::{
    AtlasKey, AtlasTextureId, AtlasTextureKind, AtlasTile, Bounds, DevicePixels, PlatformAtlas,
    Point, Size, platform::AtlasTextureList,
};
use anyhow::Result;
use collections::FxHashMap;
use etagere::BucketedAtlasAllocator;
use parking_lot::Mutex;
use std::{borrow::Cow, ops, sync::Arc};

pub(crate) struct WgpuAtlas(Mutex<WgpuAtlasState>);

unsafe impl Send for WgpuAtlas {}
unsafe impl Sync for WgpuAtlas {}

struct PendingUpload {
    id: AtlasTextureId,
    bounds: Bounds<DevicePixels>,
    data: Vec<u8>,
}

struct WgpuAtlasState {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    storage: WgpuAtlasStorage,
    tiles_by_key: FxHashMap<AtlasKey, AtlasTile>,
    uploads: Vec<PendingUpload>,
}

pub struct WgpuTextureInfo {
    pub texture_view: wgpu::TextureView,
}

impl WgpuAtlas {
    pub(crate) fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        WgpuAtlas(Mutex::new(WgpuAtlasState {
            device,
            queue,
            storage: WgpuAtlasStorage::default(),
            tiles_by_key: Default::default(),
            uploads: Vec::new(),
        }))
    }

    pub fn before_frame(&self) {
        let mut lock = self.0.lock();
        lock.flush_uploads();
    }

    pub fn get_texture_info(&self, id: AtlasTextureId) -> WgpuTextureInfo {
        let lock = self.0.lock();
        let texture = &lock.storage[id];
        WgpuTextureInfo {
            texture_view: texture.texture.create_view(&wgpu::TextureViewDescriptor::default()),
        }
    }

    pub fn get_texture_view(&self, id: AtlasTextureId) -> wgpu::TextureView {
        let lock = self.0.lock();
        let texture = &lock.storage[id];
        texture.texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

impl PlatformAtlas for WgpuAtlas {
    fn get_or_insert_with<'a>(
        &self,
        key: &AtlasKey,
        build: &mut dyn FnMut() -> Result<Option<(Size<DevicePixels>, Cow<'a, [u8]>)>>,
    ) -> Result<Option<AtlasTile>> {
        let mut lock = self.0.lock();
        if let Some(tile) = lock.tiles_by_key.get(key) {
            Ok(Some(tile.clone()))
        } else {
            let Some((size, bytes)) = build()? else {
                return Ok(None);
            };
            let tile = lock.allocate(size, key.texture_kind());
            lock.upload_texture(tile.texture_id, tile.bounds, &bytes);
            lock.tiles_by_key.insert(key.clone(), tile.clone());
            Ok(Some(tile))
        }
    }

    fn remove(&self, key: &AtlasKey) {
        let mut lock = self.0.lock();

        let Some(id) = lock.tiles_by_key.remove(key).map(|tile| tile.texture_id) else {
            return;
        };

        let storage = &mut lock.storage[id.kind];
        let Some(texture_slot) = storage.textures.get_mut(id.index as usize) else {
            return;
        };

        if let Some(texture) = texture_slot.as_mut() {
            texture.decrement_ref_count();
            if texture.is_unreferenced() {
                storage.free_list.push(id.index as usize);
                texture_slot.take();
            }
        }
    }
}

impl WgpuAtlasState {
    fn allocate(&mut self, size: Size<DevicePixels>, texture_kind: AtlasTextureKind) -> AtlasTile {
        {
            let textures = &mut self.storage[texture_kind];

            if let Some(tile) = textures
                .iter_mut()
                .rev()
                .find_map(|texture| texture.allocate(size))
            {
                return tile;
            }
        }

        let texture = self.push_texture(size, texture_kind);
        texture.allocate(size).expect("newly created texture should have space")
    }

    fn push_texture(
        &mut self,
        min_size: Size<DevicePixels>,
        kind: AtlasTextureKind,
    ) -> &mut WgpuAtlasTexture {
        const DEFAULT_ATLAS_SIZE: Size<DevicePixels> = Size {
            width: DevicePixels(1024),
            height: DevicePixels(1024),
        };

        let size = Size {
            width: min_size.width.max(DEFAULT_ATLAS_SIZE.width),
            height: min_size.height.max(DEFAULT_ATLAS_SIZE.height),
        };

        let format = match kind {
            AtlasTextureKind::Monochrome => wgpu::TextureFormat::R8Unorm,
            AtlasTextureKind::Polychrome => wgpu::TextureFormat::Bgra8Unorm,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas"),
            size: wgpu::Extent3d {
                width: size.width.0 as u32,
                height: size.height.0 as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_list = &mut self.storage[kind];
        let index = texture_list.free_list.pop();

        let atlas_texture = WgpuAtlasTexture {
            id: AtlasTextureId {
                index: index.unwrap_or(texture_list.textures.len()) as u32,
                kind,
            },
            allocator: BucketedAtlasAllocator::new(size.into()),
            format,
            texture,
            live_atlas_keys: 0,
        };

        if let Some(ix) = index {
            texture_list.textures[ix] = Some(atlas_texture);
            texture_list.textures.get_mut(ix).unwrap().as_mut().unwrap()
        } else {
            texture_list.textures.push(Some(atlas_texture));
            texture_list.textures.last_mut().unwrap().as_mut().unwrap()
        }
    }

    fn upload_texture(&mut self, id: AtlasTextureId, bounds: Bounds<DevicePixels>, bytes: &[u8]) {
        self.uploads.push(PendingUpload {
            id,
            bounds,
            data: bytes.to_vec(),
        });
    }

    fn flush_uploads(&mut self) {
        for upload in self.uploads.drain(..) {
            let texture = &self.storage[upload.id];
            let bytes_per_pixel = texture.bytes_per_pixel() as u32;

            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: upload.bounds.origin.x.0 as u32,
                        y: upload.bounds.origin.y.0 as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &upload.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.bounds.size.width.0 as u32 * bytes_per_pixel),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: upload.bounds.size.width.0 as u32,
                    height: upload.bounds.size.height.0 as u32,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}

#[derive(Default)]
struct WgpuAtlasStorage {
    monochrome_textures: AtlasTextureList<WgpuAtlasTexture>,
    polychrome_textures: AtlasTextureList<WgpuAtlasTexture>,
}

impl ops::Index<AtlasTextureKind> for WgpuAtlasStorage {
    type Output = AtlasTextureList<WgpuAtlasTexture>;
    fn index(&self, kind: AtlasTextureKind) -> &Self::Output {
        match kind {
            AtlasTextureKind::Monochrome => &self.monochrome_textures,
            AtlasTextureKind::Polychrome => &self.polychrome_textures,
        }
    }
}

impl ops::IndexMut<AtlasTextureKind> for WgpuAtlasStorage {
    fn index_mut(&mut self, kind: AtlasTextureKind) -> &mut Self::Output {
        match kind {
            AtlasTextureKind::Monochrome => &mut self.monochrome_textures,
            AtlasTextureKind::Polychrome => &mut self.polychrome_textures,
        }
    }
}

impl ops::Index<AtlasTextureId> for WgpuAtlasStorage {
    type Output = WgpuAtlasTexture;
    fn index(&self, id: AtlasTextureId) -> &Self::Output {
        let textures = match id.kind {
            AtlasTextureKind::Monochrome => &self.monochrome_textures,
            AtlasTextureKind::Polychrome => &self.polychrome_textures,
        };
        textures[id.index as usize].as_ref().expect("texture should exist")
    }
}

struct WgpuAtlasTexture {
    id: AtlasTextureId,
    allocator: BucketedAtlasAllocator,
    texture: wgpu::Texture,
    format: wgpu::TextureFormat,
    live_atlas_keys: u32,
}

impl WgpuAtlasTexture {
    fn allocate(&mut self, size: Size<DevicePixels>) -> Option<AtlasTile> {
        let allocation = self.allocator.allocate(size.into())?;
        let tile = AtlasTile {
            texture_id: self.id,
            tile_id: allocation.id.into(),
            padding: 0,
            bounds: Bounds {
                origin: allocation.rectangle.min.into(),
                size,
            },
        };
        self.live_atlas_keys += 1;
        Some(tile)
    }

    fn bytes_per_pixel(&self) -> u8 {
        match self.format {
            wgpu::TextureFormat::R8Unorm => 1,
            wgpu::TextureFormat::Bgra8Unorm => 4,
            _ => 4,
        }
    }

    fn decrement_ref_count(&mut self) {
        self.live_atlas_keys = self.live_atlas_keys.saturating_sub(1);
    }

    fn is_unreferenced(&self) -> bool {
        self.live_atlas_keys == 0
    }
}

impl From<Size<DevicePixels>> for etagere::Size {
    fn from(size: Size<DevicePixels>) -> Self {
        etagere::Size::new(size.width.into(), size.height.into())
    }
}

impl From<etagere::Point> for Point<DevicePixels> {
    fn from(value: etagere::Point) -> Self {
        Point {
            x: DevicePixels::from(value.x),
            y: DevicePixels::from(value.y),
        }
    }
}

impl From<etagere::Size> for Size<DevicePixels> {
    fn from(size: etagere::Size) -> Self {
        Size {
            width: DevicePixels::from(size.width),
            height: DevicePixels::from(size.height),
        }
    }
}

impl From<etagere::Rectangle> for Bounds<DevicePixels> {
    fn from(rectangle: etagere::Rectangle) -> Self {
        Bounds {
            origin: rectangle.min.into(),
            size: rectangle.size().into(),
        }
    }
}
