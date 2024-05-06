use std::borrow::Cow;

/// An action logged by a thread that is observing lock acquisition order.
///
/// Each thread's log file is a sequence of these enums, serialized
/// using the [`ron`] crate, one action per line.
///
/// Lock observation cannot assume that there will be any convenient
/// finalization point before the program exits, so in practice,
/// actions must be written immediately when they occur. This means we
/// can't, say, accumulate tables and write them out when they're
/// complete. The `lock-analyzer` binary is then responsible for
/// consolidating the data into a single table of observed transitions.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) enum Action<LocationAddress> {
    /// A location that we will refer to in later actions.
    ///
    /// We write one of these events the first time we see a
    /// particular `Location`. Treating this as a separate action
    /// simply lets us avoid repeating the content over and over
    /// again in every [`Acquisition`] action.
    ///
    /// [`Acquisition`]: Action::Acquisition
    Location {
        address: LocationAddress,
        file: Cow<'static, str>,
        line: u32,
        column: u32,
    },

    /// A lock rank that we will refer to in later actions.
    ///
    /// We write out one these events for every lock rank at the
    /// beginning of each thread's log file. Treating this as a
    /// separate action simply lets us avoid repeating the names over
    /// and over again in every [`Acquisition`] action.
    ///
    /// [`Acquisition`]: Action::Acquisition
    Rank {
        bit: u32,
        member_name: Cow<'static, str>,
        const_name: Cow<'static, str>,
    },

    /// An attempt to acquire a lock while holding another lock.
    Acquisition {
        /// The number of the already acquired lock's rank.
        older_rank: u32,

        /// The source position at which we acquired it. Specifically,
        /// its `Location`'s address, as an integer.
        older_location: LocationAddress,

        /// The number of the rank of the lock we are acquiring.
        newer_rank: u32,

        /// The source position at which we are acquiring it.
        /// Specifically, its `Location`'s address, as an integer.
        newer_location: LocationAddress,
    },
}
