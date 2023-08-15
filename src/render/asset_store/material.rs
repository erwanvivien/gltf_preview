use gltf::material::NormalTexture;
use gltf::material::OcclusionTexture;
use gltf::texture::Info;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlphaMode {
    Opaque = 0,
    Mask = 1,
    Blend = 2,
}

impl Into<u32> for AlphaMode {
    fn into(self) -> u32 {
        self as u32
    }
}

impl From<gltf::material::AlphaMode> for AlphaMode {
    fn from(value: gltf::material::AlphaMode) -> AlphaMode {
        use gltf::material::AlphaMode::{Blend, Mask, Opaque};
        match value {
            Opaque => AlphaMode::Opaque,
            Mask => AlphaMode::Mask,
            Blend => AlphaMode::Blend,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TextureInfo {
    pub texture_index: usize,
    pub tex_index: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct MetallicRoughness {
    metallic: f32,
    roughness: f32,
    metallic_roughness_texture: Option<TextureInfo>,
}

#[derive(Clone, Debug)]
pub struct Material {
    #[cfg(feature = "debug_gltf")]
    pub name: Option<String>,
    pub color: [f32; 4],
    pub emissive: [f32; 3],
    pub occlusion: f32,
    pub color_texture: Option<TextureInfo>,
    pub emissive_texture: Option<TextureInfo>,
    pub normals_texture: Option<TextureInfo>,
    pub occlusion_texture: Option<TextureInfo>,
    pub metallic_roughness: MetallicRoughness,
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
}

impl<'a> From<gltf::Material<'a>> for Material {
    fn from(material: gltf::Material) -> Material {
        #[cfg(feature = "debug_gltf")]
        log::info!("  Material#{:?}: {:?}", material.index(), material.name());

        let pbr = material.pbr_metallic_roughness();

        let color = pbr.base_color_factor();
        let emissive = material.emissive_factor();

        let color_texture = pbr.base_color_texture();
        let color_texture = get_texture(color_texture);
        let emissive_texture = get_texture(material.emissive_texture());
        let normals_texture = get_normals_texture(material.normal_texture());
        let (occlusion, occlusion_texture) = get_occlusion(material.occlusion_texture());

        let metallic_roughness = MetallicRoughness {
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            metallic_roughness_texture: get_texture(pbr.metallic_roughness_texture()),
        };

        let alpha_mode = material.alpha_mode().into();
        let alpha_cutoff = material.alpha_cutoff().unwrap_or(0.5);

        let double_sided = material.double_sided();

        Material {
            #[cfg(feature = "debug_gltf")]
            name: material.name().map(ToOwned::to_owned),
            color,
            emissive,
            occlusion,
            color_texture,
            emissive_texture,
            normals_texture,
            occlusion_texture,
            metallic_roughness,
            alpha_mode,
            alpha_cutoff,
            double_sided,
        }
    }
}

fn get_texture(texture_info: Option<Info>) -> Option<TextureInfo> {
    texture_info.map(|tex_info| TextureInfo {
        texture_index: tex_info.texture().source().index(),
        tex_index: tex_info.tex_coord(),
    })
}

fn get_normals_texture(texture_info: Option<NormalTexture>) -> Option<TextureInfo> {
    texture_info.map(|tex_info| TextureInfo {
        texture_index: tex_info.texture().source().index(),
        tex_index: tex_info.tex_coord(),
    })
}

fn get_occlusion(texture_info: Option<OcclusionTexture>) -> (f32, Option<TextureInfo>) {
    let strength = texture_info
        .as_ref()
        .map_or(0.0, |tex_info| tex_info.strength());

    let texture = texture_info.map(|tex_info| TextureInfo {
        texture_index: tex_info.texture().source().index(),
        tex_index: tex_info.tex_coord(),
    });

    (strength, texture)
}
