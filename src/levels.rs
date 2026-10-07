//! The five levels of compatibility. See `spec/05-compatibility.md` of tamnd/rupg.

/// One level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Level {
    /// L1 to L5.
    pub(crate) id: &'static str,
    /// What the level covers.
    pub(crate) name: &'static str,
}

/// The levels, in order. Each level has a denominator that is counted at the pin.
pub(crate) const LEVELS: [Level; 5] = [
    Level { id: "L1", name: "Connect: every protocol trace is byte-identical to the oracle" },
    Level { id: "L2", name: "Introspect: every catalog row and column is equal to the oracle" },
    Level { id: "L3", name: "Query: the same regression pass set as the oracle" },
    Level { id: "L4", name: "Behave: the same isolation outcomes and the same errors" },
    Level { id: "L5", name: "Drop-in: the same pass rate for each client suite" },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_levels_in_order() {
        let ids: Vec<&str> = LEVELS.iter().map(|l| l.id).collect();
        assert_eq!(ids, ["L1", "L2", "L3", "L4", "L5"]);
    }
}
