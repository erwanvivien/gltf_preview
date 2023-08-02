use crate::render::asset_store::{material::AlphaMode, Model};

pub struct AssetRegistry {
    pub opaque_models: Vec<Model>,
    pub transparent_models: Vec<Model>,
}

impl AssetRegistry {
    pub async fn new<P: AsRef<std::path::Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gltf_paths: &[P],
    ) -> AssetRegistry {
        let mut opaque_models = Vec::new();
        let mut transparent_models = Vec::new();

        for path in gltf_paths {
            let model = Model::from_path(path, device, queue).await;
            #[cfg(feature = "debug_gltf")]
            let model = model.expect(&format!("Failed to load model {}", path.as_ref().display()));
            #[cfg(not(feature = "debug_gltf"))]
            let model = model.expect("Failed to load model");

            let transparent = model
                .packed_primitives
                .per_primitives
                .iter()
                .any(|p| p.material.alpha_mode != AlphaMode::Opaque);

            if transparent {
                transparent_models.push(model);
            } else {
                opaque_models.push(model);
            }
        }

        Self {
            opaque_models,
            transparent_models,
        }
    }
}
