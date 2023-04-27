use crate::asset::{Asset, AssetId};

pub struct CpuTexture {
    pub id: AssetId,
    pub data: Box<dyn TextureData>,
    pub sampler_info: SamplerInfo,
}

pub trait TextureData: Sync + Send {
    fn dimensions(&self) -> (u32, u32);
    fn format(&self) -> &TextureFormat;
    fn bytes(&self) -> &[u8];
}

impl Asset for CpuTexture {
    fn id(&self) -> AssetId {
        self.id
    }
}

#[allow(non_camel_case_types)]
/// A list of the more common texture formats that we actually support.
pub enum TextureFormat {
    // TODO: Where are the sRGB formats?
    /// 8 bit texture, 1 channel, normalized color space
    R8_UNORM,
    R8G8_UNORM,
    R8G8B8A8_UNORM,
    R16_UNORM,
    R16G16_UNORM,
    R16G16B16A16_UNORM,
    R32G32B32A32_SFLOAT,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SamplerInfo {
    pub min_filter: Filter,
    pub mag_filter: Filter,
    pub address_mode: [AddressMode; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

/// A simple CPU texture implementation
/// A proper loader can supply an optimised TextureData variant instead, like one that uses a zero-copy approach.
pub struct BytesTextureData {
    pub dimensions: (u32, u32),
    pub format: TextureFormat,
    pub bytes: Vec<u8>,
}

impl BytesTextureData {
    pub fn new(dimensions: (u32, u32), format: TextureFormat, bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            dimensions,
            format,
        }
    }
}

impl TextureData for BytesTextureData {
    fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    fn format(&self) -> &TextureFormat {
        &self.format
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
