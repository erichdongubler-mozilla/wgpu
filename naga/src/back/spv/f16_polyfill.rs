/*!
This module provides functionality polyfills `f16` input/output variables when the
`StorageInputOutput16` capability is not available or disabled.

It works by:

1. Declaring `f16` I/O variables as `f32` in SPIR-V
2. Converting between `f16` and `f32` at runtime using `OpFConvert`
3. Maintaining mappings to track which variables need conversion
*/

use crate::back::spv::{recyclable::Recyclable, Instruction, LocalType, NumericType, Word};
use alloc::vec::Vec;

/// Returns a `LocalType` converting the given `ty_inner` scalar elements from `f16` to `f32`. If
/// the `ty_inner` is not an `f16` scalar or vector of `f16` scalars, returns `None`.
fn f32_local_ty(ty_inner: &crate::TypeInner) -> Option<LocalType> {
    use crate::TypeInner;

    let is_f16 =
        |scalar: &crate::Scalar| scalar.kind == crate::ScalarKind::Float && scalar.width == 2;

    match *ty_inner {
        TypeInner::Scalar(ref s) if is_f16(s) => {
            Some(LocalType::Numeric(NumericType::Scalar(crate::Scalar::F32)))
        }
        TypeInner::Vector { size, scalar } if is_f16(&scalar) => {
            Some(LocalType::Numeric(NumericType::Vector {
                size,
                scalar: crate::Scalar::F32,
            }))
        }
        _ => None,
    }
}

pub(in crate::back::spv) fn emit_f16_to_f32_conversion(
    f16_value_id: Word,
    f32_type_id: Word,
    converted_id: Word,
    body: &mut Vec<Instruction>,
) {
    body.push(Instruction::unary(
        spirv::Op::FConvert,
        f32_type_id,
        converted_id,
        f16_value_id,
    ));
}

pub(in crate::back::spv) fn emit_f32_to_f16_conversion(
    f32_value_id: Word,
    f16_type_id: Word,
    converted_id: Word,
    body: &mut Vec<Instruction>,
) {
    body.push(Instruction::unary(
        spirv::Op::FConvert,
        f16_type_id,
        converted_id,
        f32_value_id,
    ));
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::back::spv) enum F16StorageIoKind {
    #[default]
    Disabled,
    Polyfill,
    Native,
}

impl F16StorageIoKind {
    pub fn init(self) -> F16StorageIo {
        match self {
            Self::Disabled => F16StorageIo::Disabled,
            Self::Polyfill => F16StorageIo::Polyfill(Polyfill::new()),
            Self::Native => F16StorageIo::Native,
        }
    }
}

/// Manages `f16` storage I/O polyfill state and operations.
#[derive(Clone, Debug)]
pub(in crate::back::spv) enum F16StorageIo {
    Disabled,
    Polyfill(Polyfill),
    Native,
}

impl F16StorageIo {
    pub fn native() -> Self {
        Self::Native
    }

    #[track_caller]
    fn check_not_disabled(&self) {
        if let Self::Disabled = self {
            unreachable!("internal error: `f16` storage I/O was expected to be disabled")
        }
    }

    pub fn register_variable(&mut self, variable_id: Word, f32_type_id: Word, f16_type_id: Word) {
        if let Self::Polyfill(polyfill) = self {
            polyfill.register_variable(variable_id, f32_type_id, f16_type_id);
        }
    }

    pub fn polyfill_info(&self, variable_id: Word) -> Option<(Word, Word)> {
        if let Self::Polyfill(polyfill) = self {
            polyfill.map_polyfilled_to_f32(variable_id)
        } else {
            None
        }
    }

    #[track_caller]
    pub(crate) fn capabilities(&self) -> impl Iterator<Item = spirv::Capability> {
        self.check_not_disabled();
        let storage_io = match self {
            Self::Disabled => unreachable!(),
            Self::Polyfill(..) => &[][..],
            Self::Native => &[spirv::Capability::StorageInputOutput16],
        }
        .iter()
        .copied();
        [
            spirv::Capability::Float16,
            spirv::Capability::StorageBuffer16BitAccess,
            spirv::Capability::UniformAndStorageBuffer16BitAccess,
        ]
        .into_iter()
        .chain(storage_io)
    }

    #[track_caller]
    pub(crate) fn is_f16(&self, ty_inner: &crate::TypeInner) -> Option<LocalType> {
        f32_local_ty(ty_inner).inspect(|_| self.check_not_disabled())
    }
}

impl Recyclable for F16StorageIo {
    fn recycle(self) -> Self {
        match self {
            Self::Disabled => Self::Disabled,
            Self::Polyfill(polyfill) => Self::Polyfill(polyfill.recycle()),
            Self::Native => Self::Native,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(in crate::back::spv) struct Polyfill {
    variable_map: crate::FastHashMap<Word, (Word, Word)>,
}

impl Polyfill {
    pub fn new() -> Self {
        Self {
            variable_map: crate::FastHashMap::default(),
        }
    }

    pub fn register_variable(&mut self, variable_id: Word, f32_type_id: Word, f16_type_id: Word) {
        self.variable_map
            .insert(variable_id, (f32_type_id, f16_type_id));
    }

    pub fn map_polyfilled_to_f32(&self, variable_id: Word) -> Option<(Word, Word)> {
        self.variable_map.get(&variable_id).copied()
    }
}

impl Recyclable for Polyfill {
    fn recycle(mut self) -> Self {
        self.variable_map = self.variable_map.recycle();
        self
    }
}
