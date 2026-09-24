/// A "poisonable" value that indicates the use of `T` has been tainted in some way.
#[derive(Clone, Copy, Debug)]
pub(in crate::vulkan::descriptor) enum Poisonable<T> {
    Ready(T),
    Poisoned(T),
}

impl<T> Poisonable<T> {
    fn ready_result(self) -> Result<T, PoisonedError<T>> {
        match self {
            Self::Ready(ready) => Ok(ready),
            Self::Poisoned(poisoned) => Err(PoisonedError(poisoned)),
        }
    }

    /// Convert `self` to [`Self::Poisoned`].
    pub fn poison(self) -> Self {
        match self {
            Self::Ready(ready) | Self::Poisoned(ready) => Self::Poisoned(ready),
        }
    }

    /// Check if `self` is [`Self::Poisoned`].
    pub fn is_poisoned(&self) -> bool {
        matches!(self, Self::Poisoned(_))
    }
}

impl<T> Poisonable<T>
where
    T: core::fmt::Debug,
{
    /// Extract [`Self::Ready`]'s value, panicking if we are [`Self::Poisoned`].
    #[track_caller]
    pub fn ready(self) -> T {
        self.ready_result().unwrap()
    }
}

/// An error returned by [`Poisonable::ready_result`].
#[derive(Debug)]
struct PoisonedError<T>(T);
