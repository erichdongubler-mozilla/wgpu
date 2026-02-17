use thiserror::Error;
use wgt::BufferAddress;

use crate::command::{CopySide, TransferError};

/// Error encountered while checking offsets against a buffer.
#[derive(Clone, Debug, Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum BufferRegionOverrunError {
    #[error("start offset ({offset}) is out-of-bounds for buffer of size {buffer_size}")]
    StartOffset {
        offset: BufferAddress,
        buffer_size: BufferAddress,
    },
    #[error(
        "end offset (start at {} + size of {}) is out-of-bounds for buffer of size {}",
        offset,
        size,
        buffer_size
    )]
    EndOffset {
        offset: BufferAddress,
        size: BufferAddress,
        buffer_size: BufferAddress,
    },
}

impl BufferRegionOverrunError {
    /// Returns the end offset if everything is validated.
    ///
    /// TODO: more
    pub(crate) fn check(
        start_offset: BufferAddress,
        size: BufferAddress,
        buffer_size: BufferAddress,
    ) -> Result<BufferAddress, Self> {
        Self::check_start(start_offset, buffer_size)?.check_end(size)
    }

    pub(crate) fn check_start(
        start_offset: BufferAddress,
        buffer_size: BufferAddress,
    ) -> Result<BufferOverrunEndOffsetChecker, Self> {
        if start_offset >= buffer_size {
            return Err(Self::StartOffset {
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
            return Err(BufferRegionOverrunError::EndOffset {
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
