#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderKinds(pub u32);

impl ShaderKinds {
    pub const NONE: ShaderKinds = ShaderKinds(0);
    pub const POSITION: ShaderKinds = ShaderKinds(1u32 << 0);
    pub const NORMAL: ShaderKinds = ShaderKinds(1u32 << 1);
    pub const TEX_COORD_0: ShaderKinds = ShaderKinds(1u32 << 2);
    pub const TEX_COORD_1: ShaderKinds = ShaderKinds(1u32 << 3);
    pub const TANGENT: ShaderKinds = ShaderKinds(1u32 << 4);
    pub const WEIGHT: ShaderKinds = ShaderKinds(1u32 << 5);
    pub const JOINT: ShaderKinds = ShaderKinds(1u32 << 6);
    pub const COLOR: ShaderKinds = ShaderKinds(1u32 << 7);
}

impl std::fmt::Debug for ShaderKinds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShaderKinds(")?;

        let mut kinds = Vec::<&str>::new();
        for kind in Self::all() {
            if *self & kind != ShaderKinds::NONE {
                kinds.push(kind.into());
            }
        }

        let kinds_str: String = if kinds.is_empty() {
            String::from("NONE")
        } else {
            kinds.join(" | ")
        };

        f.write_str(&kinds_str)?;
        f.write_str(")")?;

        Ok(())
    }
}

impl ShaderKinds {
    #[rustfmt::skip]
    #[inline]
    const fn all() -> [Self; 8] {
        [
            Self::POSITION, Self::NORMAL, Self::TEX_COORD_0,
            Self::TEX_COORD_1, Self::TANGENT, Self::WEIGHT,
            Self::JOINT, Self::COLOR
        ]
    }

    fn all_flags() -> ShaderKinds {
        const ZERO: ShaderKinds = ShaderKinds::NONE;
        Self::all().iter().fold(ZERO, |acc, &kind| acc | kind)
    }

    pub fn is_none(self) -> bool {
        self == Self::NONE
    }

    pub fn is_position(self) -> bool {
        self & Self::POSITION != Self::NONE
    }

    pub fn is_normal(self) -> bool {
        self & Self::NORMAL != Self::NONE
    }

    pub fn is_tex_coord0(self) -> bool {
        self & Self::TEX_COORD_0 != Self::NONE
    }

    pub fn is_tex_coord1(self) -> bool {
        self & Self::TEX_COORD_1 != Self::NONE
    }

    pub fn is_tangent(self) -> bool {
        self & Self::TANGENT != Self::NONE
    }

    pub fn is_weight(self) -> bool {
        self & Self::WEIGHT != Self::NONE
    }

    pub fn is_joint(self) -> bool {
        self & Self::JOINT != Self::NONE
    }

    pub fn is_color(self) -> bool {
        self & Self::COLOR != Self::NONE
    }

    pub fn is_tex_coord(self) -> bool {
        self.is_tex_coord0() || self.is_tex_coord1()
    }
}

impl std::ops::BitOr for ShaderKinds {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let bitwise_or = self.0 | rhs.0;
        // SAFETY: We know that the result of the or is a valid ShaderInputs
        // because or is u8 and ShaderInputs is a bitfield of u8
        unsafe { std::mem::transmute(bitwise_or) }
    }
}

impl std::ops::BitAnd for ShaderKinds {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let bitwise_and = self.0 & rhs.0;
        // SAFETY: We know that the result of the and is a valid ShaderInputs
        // because and is u8 and ShaderInputs is a bitfield of u8
        unsafe { std::mem::transmute(bitwise_and) }
    }
}

impl Into<&'static str> for ShaderKinds {
    fn into(self) -> &'static str {
        match self {
            ShaderKinds::NONE => "NONE",

            ShaderKinds::POSITION => "POSITION",
            ShaderKinds::NORMAL => "NORMAL",
            ShaderKinds::TEX_COORD_0 => "TEX_COORD_0",
            ShaderKinds::TEX_COORD_1 => "TEX_COORD_1",
            ShaderKinds::TANGENT => "TANGENT",
            ShaderKinds::WEIGHT => "WEIGHTS",
            ShaderKinds::JOINT => "JOINTS",
            ShaderKinds::COLOR => "COLOR",

            _ => unreachable!(),
        }
    }
}
