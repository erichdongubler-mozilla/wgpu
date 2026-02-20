use core::{
    fmt::{self, Display, Formatter},
    ops::Range,
};

use thiserror::Error;
use wgt::BufferAddress;

use crate::command::{CopySide, TransferError};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BufferRange {
    pub start_offset: BufferAddress,
    pub size: BufferAddress,
}

#[derive(Clone, Debug, Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BufferAvailability {
    ActiveRange(Range<BufferAddress>),
    Whole { buffer_size: BufferAddress },
}

impl BufferAvailability {
    /// Returns the end offset if everything is validated.
    ///
    /// TODO: more
    pub(crate) fn check(
        &self,
        start_offset: BufferAddress,
        size: BufferAddress,
    ) -> Result<BufferAddress, BufferRangeOutOfBoundsError> {
        self.check_start(start_offset)?.check_end(size)
    }

    pub(crate) fn check_start(
        &self,
        start_offset: BufferAddress,
    ) -> Result<BufferOverrunEndOffsetChecker, BufferRangeOutOfBoundsError> {
        if start_offset > buffer_size {
            return Err(Self::StartOffsetOverrun {
                offset: start_offset,
                buffer_size,
            });
        }
        Ok(BufferOverrunEndOffsetChecker {
            start_offset,
            buffer_size,
        })
    }
}

/// Error encountered while checking offsets against a buffer.
#[derive(Clone, Debug, Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BufferRangeOutOfBoundsError {
    pub range: BufferRange,
    pub availability: BufferAvailability,
    pub kind: BufferRangeOutOfBoundsErrorKind,
}

impl Display for BufferRangeOutOfBoundsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            range: BufferRange { start_offset, size },
            availability,
            kind,
        } = self;
        let offset_at_start: &dyn Fn(&mut Formatter<'_>) -> fmt::Result =
            &|f| write!(f, "start offset ({start_offset})");
        let offset_at_end: &dyn Fn(&mut Formatter<'_>) -> fmt::Result =
            &|f| write!(f, "start offset ({start_offset}) + size ({size})");
        let (write_offset, direction_desc) = match kind {
            BufferRangeOutOfBoundsErrorKind::StartUnderrun => (offset_at_start, "under"),
            BufferRangeOutOfBoundsErrorKind::StartOverrun => (offset_at_start, "over"),
            BufferRangeOutOfBoundsErrorKind::EndOverrun => (offset_at_end, "over"),
        };
        (write_offset)(f)?;
        write!(f, " would {}run ", direction_desc)?;
        match availability {
            BufferAvailability::ActiveRange(buffer_range) => {
                write!(f, "active buffer range {buffer_range:?}")
            }
            BufferAvailability::Whole { buffer_size } => {
                write!(f, "buffer of size {buffer_size}")
            }
        }?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum BufferRangeOutOfBoundsErrorKind {
    StartUnderrun,
    StartOverrun,
    EndOverrun,
}

/// A
#[derive(Clone, Copy, Debug)]
pub(crate) struct BufferOverrunEndOffsetChecker {
    start_offset: BufferAddress,
    buffer_size: BufferAddress,
}

impl BufferOverrunEndOffsetChecker {
    pub(crate) fn check_end(
        self,
        size: BufferAddress,
    ) -> Result<BufferAddress, BufferRegionOverrunError> {
        let Self {
            start_offset,
            buffer_size,
        } = self;

        // NOTE: Should never underflow because of our earlier check.
        if size > buffer_size - start_offset {
            return Err(BufferRegionOverrunError::EndOffsetOverrun {
                offset: start_offset,
                size,
                buffer_size,
            });
        }
        // NOTE: Should never overflow because of our earlier check.
        Ok(start_offset + size)
    }
}

impl BufferRegionOverrunError {
    pub(crate) fn to_transfer_error(&self, side: CopySide) -> TransferError {
        todo!()
    }
}
