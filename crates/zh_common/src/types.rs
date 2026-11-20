//! Core deterministic types matching C++ `BaseType.h`, `GameType.h`, and `GameCommon.h`.

pub type Int = i32;
pub type UnsignedInt = u32;
pub type Short = i16;
pub type UnsignedShort = u16;
pub type Byte = i8;
pub type UnsignedByte = u8;
pub type Bool = u8;
pub type Real = f32;
pub type XferVersion = u8;

pub const FALSE: Bool = 0;
pub const TRUE: Bool = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ObjectId(pub i32);

impl ObjectId {
    pub const INVALID: Self = Self(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct TeamId(pub i32);

impl TeamId {
    pub const INVALID: Self = Self(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct DrawableId(pub i32);

impl DrawableId {
    pub const INVALID: Self = Self(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct NameKeyType(pub i32);

impl NameKeyType {
    pub const INVALID: Self = Self(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UpdateSleepTime(pub u32);

impl UpdateSleepTime {
    pub const NONE: Self = Self(1);
    pub const FOREVER: Self = Self(0x3fff_ffff);
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum VeterancyLevel {
    #[default]
    Regular = 0,
    Veteran = 1,
    Elite = 2,
    Heroic = 3,
    Invalid = 4,
}

impl VeterancyLevel {
    pub const COUNT: i32 = 4;
    pub const FIRST: i32 = 0;
    pub const LAST: i32 = 3;

    #[inline]
    pub const fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::Regular,
            1 => Self::Veteran,
            2 => Self::Elite,
            3 => Self::Heroic,
            _ => Self::Invalid,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Relationship {
    #[default]
    Enemies = 0,
    Neutral = 1,
    Allies = 2,
}

impl Relationship {
    #[inline]
    pub const fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::Enemies,
            1 => Self::Neutral,
            2 => Self::Allies,
            _ => Self::Neutral,
        }
    }
}
