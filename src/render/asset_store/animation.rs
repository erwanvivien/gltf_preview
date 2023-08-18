use std::collections::HashSet;

use crate::render::asset_store::NodeIndex;

#[derive(Clone, Copy, Debug)]
pub enum PropertyKind {
    Translation,
    Rotation,
    Scale,
    MorphTargetWeights,
}

impl PropertyKind {
    #[inline]
    fn size(&self) -> usize {
        match self {
            PropertyKind::Translation => 3,
            PropertyKind::Rotation => 4,
            PropertyKind::Scale => 3,
            PropertyKind::MorphTargetWeights => 1,
        }
    }
}

pub enum PropertyValue {
    Translation(glam::Vec3),
    Rotation(glam::Quat),
    Scale(glam::Vec3),
    MorphTargetWeights(f32),
}

impl PropertyValue {
    pub fn kind(&self) -> PropertyKind {
        match self {
            PropertyValue::Translation(_) => PropertyKind::Translation,
            PropertyValue::Rotation(_) => PropertyKind::Rotation,
            PropertyValue::Scale(_) => PropertyKind::Scale,
            PropertyValue::MorphTargetWeights(_) => PropertyKind::MorphTargetWeights,
        }
    }
}

impl PropertyValue {
    fn from(property: PropertyKind, data: &[f32]) -> Self {
        match property {
            PropertyKind::Translation => PropertyValue::Translation(glam::Vec3::from_slice(data)),
            PropertyKind::Scale => PropertyValue::Scale(glam::Vec3::from_slice(data)),
            PropertyKind::Rotation => PropertyValue::Rotation(glam::Quat::from_slice(data)),
            PropertyKind::MorphTargetWeights => unimplemented!(),
        }
    }
}

impl From<gltf::animation::Property> for PropertyKind {
    fn from(value: gltf::animation::Property) -> Self {
        use gltf::animation::Property;
        match value {
            Property::Translation => Self::Translation,
            Property::Rotation => Self::Rotation,
            Property::Scale => Self::Scale,
            Property::MorphTargetWeights => Self::MorphTargetWeights,
        }
    }
}

/// Clone of [Interpolation](gltf::animation::Interpolation)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Interpolation {
    Linear = 1,
    Step,
    CubicSpline,
}

impl From<gltf::animation::Interpolation> for Interpolation {
    fn from(value: gltf::animation::Interpolation) -> Self {
        use gltf::animation::Interpolation;
        match value {
            Interpolation::Linear => Self::Linear,
            Interpolation::Step => Self::Step,
            Interpolation::CubicSpline => Self::CubicSpline,
        }
    }
}

#[derive(Clone)]
struct Data(Vec<f32>);

impl Data {
    fn get(&self, index: usize, property_kind: PropertyKind) -> &[f32] {
        let start_index = index * property_kind.size();
        let end_index = start_index + property_kind.size();
        &self.0[start_index..end_index]
    }
}

#[derive(Clone)]
pub struct Channel {
    pub node_index: NodeIndex,
    interpolation: Interpolation,
    times: Vec<f32>,
    duration: f32,
    property: PropertyKind,

    /// Needs to be cast as underlying Property kind (i.e. [glam::Vec3])
    /// before use
    data: Data,
}

impl Channel {
    fn parse(channel: &gltf::animation::Channel, buffers: &[gltf::buffer::Data]) -> Self {
        let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));

        let target = channel.target();
        let property: PropertyKind = target.property().into();
        let node_index = target.node().index();
        let node_index = u32::try_from(node_index).expect("Node index overflow");
        let node_index = NodeIndex(node_index);

        let sampler = channel.sampler();
        let interpolation: Interpolation = sampler.interpolation().into();

        let input_normalize = sampler.input().normalized();
        let output_normalize = sampler.output().normalized();
        let times = read_times(&reader, input_normalize);
        let data = read_outputs(&reader, output_normalize);

        // Already checked in read_times & outputs
        let duration = *times.last().unwrap();

        #[cfg(feature = "debug_gltf")]
        log::info!(
            "  Animates {:?} ({:?}) for Node#{} ({})",
            target.property(),
            interpolation,
            node_index.0,
            target.node().name().unwrap_or("None"),
        );

        Channel {
            node_index,
            interpolation,
            times,
            data: Data(data),
            duration,
            property,
        }
    }

    pub fn interpolate(&self, time_since_program_start: f32) -> PropertyValue {
        let time_mod = time_since_program_start % self.duration;

        use std::cmp::Ordering::Equal;
        let time_index = self
            .times
            .binary_search_by(|a| a.partial_cmp(&time_mod).unwrap_or(Equal));

        let Err(time_index) = time_index else {
            let data = self.data.get(time_index.unwrap(), self.property);
            return PropertyValue::from(self.property, data);
        };
        let time_index = time_index - 1;

        #[cfg(debug_assertions)]
        assert!(time_index + 1 < self.times.len());

        let first_time = self.times[time_index];
        let second_time = self.times[time_index + 1];
        let delta = second_time - first_time;

        #[cfg(debug_assertions)]
        assert!(delta >= 0.0);

        let time_ratio = (time_mod - first_time) / delta;

        // We are using linear interpolation for Translate & Scale
        // We are using spherical linear interpolation for Rotation
        // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#appendix-c-interpolation
        match self.interpolation {
            Interpolation::Step => {
                let data = self.data.get(time_index, self.property);
                PropertyValue::from(self.property, data)
            }
            Interpolation::Linear => {
                let first = self.data.get(time_index, self.property);
                let second = self.data.get(time_index + 1, self.property);

                match self.property {
                    PropertyKind::Scale | PropertyKind::Translation => {
                        let first = glam::Vec3::from_slice(first);
                        let second = glam::Vec3::from_slice(second);

                        let data = &first.lerp(second, time_ratio).to_array()[..];
                        PropertyValue::from(self.property, data)
                    }
                    PropertyKind::Rotation => {
                        let first = glam::Quat::from_slice(first);
                        let second = glam::Quat::from_slice(second);

                        let data = &first.slerp(second, time_ratio).to_array()[..];
                        PropertyValue::from(self.property, data)
                    }
                    PropertyKind::MorphTargetWeights => unimplemented!(),
                }
            }
            Interpolation::CubicSpline => todo!(),
        }
    }
}

pub struct Animation {
    #[cfg(feature = "debug_gltf")]
    pub name: Option<String>,

    pub target_nodes: HashSet<NodeIndex>,
    pub channels: Vec<Channel>,
}

impl Animation {
    pub fn parse(animation: &gltf::Animation, buffers: &[gltf::buffer::Data]) -> Self {
        #[cfg(feature = "debug_gltf")]
        log::info!(
            "Animation {} with {} channels",
            animation.name().unwrap_or("None"),
            animation.channels().count()
        );

        let mut target_nodes = HashSet::new();
        let mut channels = Vec::new();

        for channel in animation.channels() {
            let channel = Channel::parse(&channel, buffers);
            target_nodes.insert(channel.node_index);
            channels.push(channel);
        }

        Animation {
            #[cfg(feature = "debug_gltf")]
            name: animation.name().map(ToOwned::to_owned),
            channels,
            target_nodes,
        }
    }
}

// From https://github.com/adrien-ben/gltf-viewer-rs/blob/eebdd3/crates/libs/model/src/animation.rs#L464-L508
use gltf::animation::util::ReadOutputs;

fn read_times<'a, 's, F>(
    reader: &gltf::animation::Reader<'a, 's, F>,
    // TODO: Find how to normalize times
    input_normalized: bool,
) -> Vec<f32>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    let times = reader
        .read_inputs()
        .map(|times| times.collect::<Vec<_>>())
        .expect("No times were found");

    if input_normalized {
        let (min, max) = times
            .iter()
            .fold((f32::MAX, f32::MIN), |(min, max), &time| {
                (min.min(time), max.max(time))
            });
        let range = max - min;

        times
            .into_iter()
            .map(|time| (time - min) / range)
            .collect::<Vec<_>>()
    } else {
        times
    }
}

fn read_outputs<'a, 's, F>(
    reader: &gltf::animation::Reader<'a, 's, F>,
    output_normalized: bool,
) -> Vec<f32>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    use glam::Quat;

    let outputs = reader
        .read_outputs()
        .map(|outputs| match outputs {
            ReadOutputs::Rotations(rotations) => {
                let mut rotations: Vec<_> = rotations.into_f32().map(Quat::from_array).collect();

                if output_normalized {
                    for data in &mut rotations {
                        *data = data.normalize();
                    }
                }

                let rotations: Vec<_> = rotations.iter().map(Quat::to_array).collect();
                bytemuck::cast_vec(rotations)
            }
            ReadOutputs::Scales(vec3) | ReadOutputs::Translations(vec3) => {
                let mut data: Vec<_> = vec3.map(glam::Vec3::from_array).collect();

                if output_normalized {
                    for data in &mut data {
                        *data = data.normalize();
                    }
                }

                bytemuck::cast_vec(data)
            }
            ReadOutputs::MorphTargetWeights(_morph_target) => {
                // morph_target.into_f32().collect::<Vec<_>>()
                panic!("MorphTargetWeights not supported")
            }
        })
        .expect("No data was found");

    outputs
}
