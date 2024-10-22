//! `enable …;` extensions in WGSL.
//!
//! The focal point of this module is the [`EnableExtension`] API.
use crate::{front::wgsl::error::Error, Span};

/// Tracks the status of every enable extension known to Naga.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnableExtensions {
    #[cfg(test)]
    definitely_not_standard: bool,
}

impl EnableExtensions {
    pub(crate) const fn empty() -> Self {
        Self {
            #[cfg(test)]
            definitely_not_standard: false,
        }
    }

    /// Add an enable extension to the set requested by a module.
    #[allow(unreachable_code)]
    pub(crate) fn add(&mut self, ext: ImplementedEnableExtension) {
        let _field: &mut bool = match ext {
            #[cfg(test)]
            ImplementedEnableExtension::DefinitelyNotStandard => &mut self.definitely_not_standard,
        };
        *_field = true;
    }

    /// Query whether an enable extension tracked here has been requested.
    #[allow(unused)]
    pub(crate) const fn contains(&self, ext: ImplementedEnableExtension) -> bool {
        match ext {
            #[cfg(test)]
            ImplementedEnableExtension::DefinitelyNotStandard => self.definitely_not_standard,
        }
    }
}

impl Default for EnableExtensions {
    fn default() -> Self {
        Self::empty()
    }
}

/// A shader language extension not guaranteed to be present in all environments.
///
/// WGSL spec.: <https://www.w3.org/TR/WGSL/#enable-extensions-sec>
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum EnableExtension {
    #[cfg_attr(not(test), allow(unused))]
    Implemented(ImplementedEnableExtension),
    Unimplemented(UnimplementedEnableExtension),
}

impl EnableExtension {
    #[cfg(test)]
    const DEFINITELY_NOT_STANDARD: &'static str = "definitely_not_standard";
    const F16: &'static str = "f16";
    const CLIP_DISTANCES: &'static str = "clip_distances";
    const DUAL_SOURCE_BLENDING: &'static str = "dual_source_blending";

    /// Convert from a sentinel word in WGSL into its associated [`EnableExtension`], if possible.
    pub(crate) fn from_ident(word: &str, span: Span) -> Result<Self, Error<'_>> {
        Ok(match word {
            Self::F16 => Self::Unimplemented(UnimplementedEnableExtension::F16),
            Self::CLIP_DISTANCES => {
                Self::Unimplemented(UnimplementedEnableExtension::ClipDistances)
            }
            Self::DUAL_SOURCE_BLENDING => {
                Self::Unimplemented(UnimplementedEnableExtension::DualSourceBlending)
            }
            _ => return Err(Error::UnknownEnableExtension(span, word)),
        })
    }

    /// Maps this [`EnableExtension`] into the sentinel word associated with it in WGSL.
    pub const fn to_ident(self) -> &'static str {
        match self {
            Self::Implemented(kind) => match kind {
                #[cfg(test)]
                ImplementedEnableExtension::DefinitelyNotStandard => Self::DEFINITELY_NOT_STANDARD,
            },
            Self::Unimplemented(kind) => match kind {
                UnimplementedEnableExtension::F16 => Self::F16,
                UnimplementedEnableExtension::ClipDistances => Self::CLIP_DISTANCES,
                UnimplementedEnableExtension::DualSourceBlending => Self::DUAL_SOURCE_BLENDING,
            },
        }
    }
}

/// A variant of [`EnableExtension::Implemented`].
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum ImplementedEnableExtension {
    #[cfg(test)]
    DefinitelyNotStandard,
}

/// A variant of [`EnableExtension::Unimplemented`].
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum UnimplementedEnableExtension {
    /// Enables `f16`/`half` primitive support in all shader languages.
    ///
    /// In the WGSL standard, this corresponds to [`enable f16;`].
    ///
    /// [`enable f16;`]: https://www.w3.org/TR/WGSL/#extension-f16
    F16,
    /// Enables the `clip_distances` variable in WGSL.
    ///
    /// In the WGSL standard, this corresponds to [`enable clip_distances;`].
    ///
    /// [`enable clip_distances;`]: https://www.w3.org/TR/WGSL/#extension-f16
    ClipDistances,
    /// Enables the `blend_src` attribute in WGSL.
    ///
    /// In the WGSL standard, this corresponds to [`enable dual_source_blending;`].
    ///
    /// [`enable dual_source_blending;`]: https://www.w3.org/TR/WGSL/#extension-f16
    DualSourceBlending,
}

impl UnimplementedEnableExtension {
    pub(crate) const fn tracking_issue_num(self) -> u16 {
        match self {
            Self::F16 => 4384,
            Self::ClipDistances => 6236,
            Self::DualSourceBlending => 6402,
        }
    }
}
#[cfg(test)]
mod test {
    use strum::IntoEnumIterator as _;

    use super::ImplementedEnableExtension;

    fn valid() {
        for extension in ImplementedEnableExtension::iter() {}
    }

    fn unimplemented() {}
}
