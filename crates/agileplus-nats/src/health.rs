//! Health-check types for the event bus.

/// Health status of the event bus connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusHealth {
    Connected,
    Disconnected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connected_variants_equal() {
        let a = BusHealth::Connected;
        let b = BusHealth::Connected;
        assert_eq!(a, b);
    }

    #[test]
    fn disconnected_variants_equal() {
        let a = BusHealth::Disconnected;
        let b = BusHealth::Disconnected;
        assert_eq!(a, b);
    }

    #[test]
    fn connected_not_equal_to_disconnected() {
        assert_ne!(BusHealth::Connected, BusHealth::Disconnected);
    }

    #[test]
    fn debug_impl_connected() {
        let h = BusHealth::Connected;
        assert_eq!(format!("{h:?}"), "Connected");
    }

    #[test]
    fn debug_impl_disconnected() {
        let h = BusHealth::Disconnected;
        assert_eq!(format!("{h:?}"), "Disconnected");
    }

    #[test]
    fn copy_semantics() {
        let a = BusHealth::Connected;
        let b = a; // Copy
        assert_eq!(a, b);
        // Both still valid after copy
        assert_eq!(a, BusHealth::Connected);
        assert_eq!(b, BusHealth::Connected);
    }

    #[test]
    fn clone_semantics() {
        let a = BusHealth::Disconnected;
        let b = a.clone();
        assert_eq!(a, b);
    }
}
