use core::fmt::{self, Debug, Display, Formatter};

#[cfg(doc)]
#[expect(unused_imports)]
use crate::validation::StageError;

/// Max shader I/O variable deductions for vertex shader output. Used by
/// [`StageError::TooManyUserDefinedVertexOutputs`] and
/// [`StageError::VertexOutputLocationTooLarge`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaxVertexShaderOutputDeduction {
    /// When a pipeline's [`crate::pipeline::RenderPipelineDescriptor::primitive`] is set to
    /// [`wgt::PrimitiveTopology::PointList`].
    PointListPrimitiveTopology,
}

impl MaxVertexShaderOutputDeduction {
    pub fn for_variables(self) -> u32 {
        match self {
            Self::PointListPrimitiveTopology => 1,
        }
    }

    pub fn for_location(self) -> u32 {
        match self {
            Self::PointListPrimitiveTopology => 0,
        }
    }
}

/// Max shader I/O variable deductions for vertex shader output. Used by
/// [`StageError::TooManyUserDefinedFragmentInputs`] and
/// [`StageError::FragmentInputLocationTooLarge`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaxFragmentShaderInputDeduction {
    InterStageBuiltIn(InterStageFragmentInputBuiltIn),
}

impl MaxFragmentShaderInputDeduction {
    pub fn for_variables(self) -> u32 {
        match self {
            Self::InterStageBuiltIn(builtin) => match builtin {
                InterStageFragmentInputBuiltIn::FrontFacing
                | InterStageFragmentInputBuiltIn::SampleIndex
                | InterStageFragmentInputBuiltIn::SampleMask
                | InterStageFragmentInputBuiltIn::PrimitiveIndex
                | InterStageFragmentInputBuiltIn::SubgroupInvocationId
                | InterStageFragmentInputBuiltIn::SubgroupSize
                | InterStageFragmentInputBuiltIn::ViewIndex
                | InterStageFragmentInputBuiltIn::PointCoord => 1,
                InterStageFragmentInputBuiltIn::Barycentric => 3,
                InterStageFragmentInputBuiltIn::Position => 0,
            },
        }
    }

    pub fn from_inter_stage_builtin(builtin: naga::BuiltIn) -> Option<Self> {
        use naga::BuiltIn;

        Some(Self::InterStageBuiltIn(match builtin {
            BuiltIn::Position { .. } => InterStageFragmentInputBuiltIn::Position,
            BuiltIn::FrontFacing => InterStageFragmentInputBuiltIn::FrontFacing,
            BuiltIn::SampleIndex => InterStageFragmentInputBuiltIn::SampleIndex,
            BuiltIn::SampleMask => InterStageFragmentInputBuiltIn::SampleMask,
            BuiltIn::PrimitiveIndex => InterStageFragmentInputBuiltIn::PrimitiveIndex,
            BuiltIn::SubgroupSize => InterStageFragmentInputBuiltIn::SubgroupSize,
            BuiltIn::SubgroupInvocationId => InterStageFragmentInputBuiltIn::SubgroupInvocationId,
            BuiltIn::PointCoord => InterStageFragmentInputBuiltIn::PointCoord,
            BuiltIn::Barycentric { .. } => InterStageFragmentInputBuiltIn::Barycentric,
            BuiltIn::ViewIndex => InterStageFragmentInputBuiltIn::ViewIndex,
            BuiltIn::BaseInstance
            | BuiltIn::BaseVertex
            | BuiltIn::ClipDistance
            | BuiltIn::CullDistance
            | BuiltIn::InstanceIndex
            | BuiltIn::PointSize
            | BuiltIn::VertexIndex
            | BuiltIn::DrawID
            | BuiltIn::FragDepth
            | BuiltIn::GlobalInvocationId
            | BuiltIn::LocalInvocationId
            | BuiltIn::LocalInvocationIndex
            | BuiltIn::WorkGroupId
            | BuiltIn::WorkGroupSize
            | BuiltIn::NumWorkGroups
            | BuiltIn::NumSubgroups
            | BuiltIn::SubgroupId
            | BuiltIn::MeshTaskSize
            | BuiltIn::CullPrimitive
            | BuiltIn::PointIndex
            | BuiltIn::LineIndices
            | BuiltIn::TriangleIndices
            | BuiltIn::VertexCount
            | BuiltIn::Vertices
            | BuiltIn::PrimitiveCount
            | BuiltIn::Primitives
            | BuiltIn::RayInvocationId
            | BuiltIn::NumRayInvocations
            | BuiltIn::InstanceCustomData
            | BuiltIn::GeometryIndex
            | BuiltIn::WorldRayOrigin
            | BuiltIn::WorldRayDirection
            | BuiltIn::ObjectRayOrigin
            | BuiltIn::ObjectRayDirection
            | BuiltIn::RayTmin
            | BuiltIn::RayTCurrentMax
            | BuiltIn::ObjectToWorld
            | BuiltIn::WorldToObject
            | BuiltIn::HitKind => return None,
        }))
    }
}

/// A [`naga::BuiltIn`] that counts towards
/// a [`MaxFragmentShaderInputDeduction::InterStageBuiltIn`].
///
/// See also <https://www.w3.org/TR/webgpu/#inter-stage-builtins>.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterStageFragmentInputBuiltIn {
    // Standard for WebGPU
    Position,
    FrontFacing,
    SampleIndex,
    SampleMask,
    PrimitiveIndex,
    SubgroupInvocationId,
    SubgroupSize,

    // Non-standard
    PointCoord,
    Barycentric,
    ViewIndex,
}

pub(in crate::validation) fn display_deductions_as_optional_list<T>(
    deductions: &[T],
    accessor: fn(&T) -> u32,
) -> impl Display + '_
where
    T: Debug,
{
    struct DisplayFromFn<F>(F);

    impl<F> Display for DisplayFromFn<F>
    where
        F: Fn(&mut Formatter<'_>) -> fmt::Result,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let Self(inner) = self;
            inner(f)
        }
    }

    DisplayFromFn(move |f: &mut Formatter<'_>| {
        let relevant_deductions = deductions
            .iter()
            .map(|deduction| (deduction, accessor(deduction)))
            .filter(|(_, effective_deduction)| *effective_deduction > 0);
        if relevant_deductions.clone().next().is_some() {
            writeln!(f, "; note that some deductions apply during validation:")?;
            let mut wrote_something = false;
            for deduction in deductions {
                let deducted_amount = accessor(deduction);
                if deducted_amount > 0 {
                    writeln!(f, "\n- {deduction:?}: {}", accessor(deduction))?;
                    wrote_something = true;
                }
            }
            debug_assert!(
                wrote_something,
                "no substantial deductions found in error display"
            );
        }
        Ok(())
    })
}
