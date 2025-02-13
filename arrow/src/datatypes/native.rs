// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use super::DataType;
use half::f16;

mod private {
    pub trait Sealed {}
}

/// Trait expressing a Rust type that has the same in-memory representation
/// as Arrow. This includes `i16`, `f32`, but excludes `bool` (which in arrow is represented in bits).
///
/// In little endian machines, types that implement [`ArrowNativeType`] can be memcopied to arrow buffers
/// as is.
///
/// # Transmute Safety
///
/// A type T implementing this trait means that any arbitrary slice of bytes of length and
/// alignment `size_of::<T>()` can be safely interpreted as a value of that type without
/// being unsound, i.e. potentially resulting in undefined behaviour.
///
/// Note: in the case of floating point numbers this transmutation can result in a signalling
/// NaN, which, whilst sound, can be unwieldy. In general, whilst it is perfectly sound to
/// reinterpret bytes as different types using this trait, it is likely unwise. For more information
/// see [f32::from_bits] and [f64::from_bits].
///
/// Note: `bool` is restricted to `0` or `1`, and so `bool: !ArrowNativeType`
///
/// # Sealed
///
/// Due to the above restrictions, this trait is sealed to prevent accidental misuse
pub trait ArrowNativeType:
    std::fmt::Debug
    + Send
    + Sync
    + Copy
    + PartialOrd
    + std::str::FromStr
    + Default
    + private::Sealed
    + 'static
{
    /// Convert native type from usize.
    #[inline]
    fn from_usize(_: usize) -> Option<Self> {
        None
    }

    /// Convert native type to usize.
    #[inline]
    fn to_usize(&self) -> Option<usize> {
        None
    }

    /// Convert native type to isize.
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        None
    }

    /// Convert native type from i32.
    #[inline]
    fn from_i32(_: i32) -> Option<Self> {
        None
    }

    /// Convert native type from i64.
    #[inline]
    fn from_i64(_: i64) -> Option<Self> {
        None
    }

    /// Convert native type from i128.
    #[inline]
    fn from_i128(_: i128) -> Option<Self> {
        None
    }
}

/// Trait bridging the dynamic-typed nature of Arrow (via [`DataType`]) with the
/// static-typed nature of rust types ([`ArrowNativeType`]) for all types that implement [`ArrowNativeType`].
pub trait ArrowPrimitiveType: 'static {
    /// Corresponding Rust native type for the primitive type.
    type Native: ArrowNativeType;

    /// the corresponding Arrow data type of this primitive type.
    const DATA_TYPE: DataType;

    /// Returns the byte width of this primitive type.
    fn get_byte_width() -> usize {
        std::mem::size_of::<Self::Native>()
    }

    /// Returns a default value of this primitive type.
    ///
    /// This is useful for aggregate array ops like `sum()`, `mean()`.
    fn default_value() -> Self::Native {
        Default::default()
    }
}

pub(crate) mod native_op {
    use super::ArrowNativeType;
    use std::ops::{Add, Div, Mul, Sub};

    /// Trait for ArrowNativeType to provide overflow-checking and non-overflow-checking
    /// variants for arithmetic operations. For floating point types, this provides some
    /// default implementations. Integer types that need to deal with overflow can implement
    /// this trait.
    ///
    /// The APIs with `_wrapping` suffix are the variant of non-overflow-checking. If overflow
    /// occurred, they will supposedly wrap around the boundary of the type.
    ///
    /// The APIs with `_checked` suffix are the variant of overflow-checking which return `None`
    /// if overflow occurred.
    pub trait ArrowNativeTypeOp:
        ArrowNativeType
        + Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
    {
        fn add_checked(self, rhs: Self) -> Option<Self> {
            Some(self + rhs)
        }

        fn add_wrapping(self, rhs: Self) -> Self {
            self + rhs
        }

        fn sub_checked(self, rhs: Self) -> Option<Self> {
            Some(self - rhs)
        }

        fn sub_wrapping(self, rhs: Self) -> Self {
            self - rhs
        }

        fn mul_checked(self, rhs: Self) -> Option<Self> {
            Some(self * rhs)
        }

        fn mul_wrapping(self, rhs: Self) -> Self {
            self * rhs
        }

        fn div_checked(self, rhs: Self) -> Option<Self> {
            Some(self / rhs)
        }

        fn div_wrapping(self, rhs: Self) -> Self {
            self / rhs
        }
    }
}

macro_rules! native_type_op {
    ($t:tt) => {
        impl native_op::ArrowNativeTypeOp for $t {
            fn add_checked(self, rhs: Self) -> Option<Self> {
                self.checked_add(rhs)
            }

            fn add_wrapping(self, rhs: Self) -> Self {
                self.wrapping_add(rhs)
            }

            fn sub_checked(self, rhs: Self) -> Option<Self> {
                self.checked_sub(rhs)
            }

            fn sub_wrapping(self, rhs: Self) -> Self {
                self.wrapping_sub(rhs)
            }

            fn mul_checked(self, rhs: Self) -> Option<Self> {
                self.checked_mul(rhs)
            }

            fn mul_wrapping(self, rhs: Self) -> Self {
                self.wrapping_mul(rhs)
            }

            fn div_checked(self, rhs: Self) -> Option<Self> {
                self.checked_div(rhs)
            }

            fn div_wrapping(self, rhs: Self) -> Self {
                self.wrapping_div(rhs)
            }
        }
    };
}

native_type_op!(i8);
native_type_op!(i16);
native_type_op!(i32);
native_type_op!(i64);
native_type_op!(u8);
native_type_op!(u16);
native_type_op!(u32);
native_type_op!(u64);

impl native_op::ArrowNativeTypeOp for f16 {}
impl native_op::ArrowNativeTypeOp for f32 {}
impl native_op::ArrowNativeTypeOp for f64 {}

impl private::Sealed for i8 {}
impl ArrowNativeType for i8 {
    #[inline]
    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }

    #[inline]
    fn to_isize(&self) -> Option<isize> {
        num::ToPrimitive::to_isize(self)
    }
}

impl private::Sealed for i16 {}
impl ArrowNativeType for i16 {
    #[inline]
    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }

    #[inline]
    fn to_isize(&self) -> Option<isize> {
        num::ToPrimitive::to_isize(self)
    }
}

impl private::Sealed for i32 {}
impl ArrowNativeType for i32 {
    #[inline]
    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }

    #[inline]
    fn to_isize(&self) -> Option<isize> {
        num::ToPrimitive::to_isize(self)
    }

    /// Convert native type from i32.
    #[inline]
    fn from_i32(val: i32) -> Option<Self> {
        Some(val)
    }
}

impl private::Sealed for i64 {}
impl ArrowNativeType for i64 {
    #[inline]
    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }

    #[inline]
    fn to_isize(&self) -> Option<isize> {
        num::ToPrimitive::to_isize(self)
    }

    /// Convert native type from i64.
    #[inline]
    fn from_i64(val: i64) -> Option<Self> {
        Some(val)
    }
}

impl private::Sealed for i128 {}
impl ArrowNativeType for i128 {
    #[inline]
    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }

    #[inline]
    fn to_isize(&self) -> Option<isize> {
        num::ToPrimitive::to_isize(self)
    }

    /// Convert native type from i128.
    #[inline]
    fn from_i128(val: i128) -> Option<Self> {
        Some(val)
    }
}

impl private::Sealed for u8 {}
impl ArrowNativeType for u8 {
    #[inline]
    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }

    #[inline]
    fn to_isize(&self) -> Option<isize> {
        num::ToPrimitive::to_isize(self)
    }
}

impl private::Sealed for u16 {}
impl ArrowNativeType for u16 {
    #[inline]
    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }

    #[inline]
    fn to_isize(&self) -> Option<isize> {
        num::ToPrimitive::to_isize(self)
    }
}

impl private::Sealed for u32 {}
impl ArrowNativeType for u32 {
    #[inline]
    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }

    #[inline]
    fn to_isize(&self) -> Option<isize> {
        num::ToPrimitive::to_isize(self)
    }
}

impl private::Sealed for u64 {}
impl ArrowNativeType for u64 {
    #[inline]
    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }

    #[inline]
    fn to_isize(&self) -> Option<isize> {
        num::ToPrimitive::to_isize(self)
    }
}

impl ArrowNativeType for f16 {}
impl private::Sealed for f16 {}
impl ArrowNativeType for f32 {}
impl private::Sealed for f32 {}
impl ArrowNativeType for f64 {}
impl private::Sealed for f64 {}

/// Allows conversion from supported Arrow types to a byte slice.
pub trait ToByteSlice {
    /// Converts this instance into a byte slice
    fn to_byte_slice(&self) -> &[u8];
}

impl<T: ArrowNativeType> ToByteSlice for [T] {
    #[inline]
    fn to_byte_slice(&self) -> &[u8] {
        let raw_ptr = self.as_ptr() as *const T as *const u8;
        unsafe {
            std::slice::from_raw_parts(raw_ptr, self.len() * std::mem::size_of::<T>())
        }
    }
}

impl<T: ArrowNativeType> ToByteSlice for T {
    #[inline]
    fn to_byte_slice(&self) -> &[u8] {
        let raw_ptr = self as *const T as *const u8;
        unsafe { std::slice::from_raw_parts(raw_ptr, std::mem::size_of::<T>()) }
    }
}
