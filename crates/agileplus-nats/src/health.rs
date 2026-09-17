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

    #[test]
    fn enum_variants_exhaustive() {
        // Exhaustive match to ensure no missing variants in future.
        let vals = [BusHealth::Connected, BusHealth::Disconnected];
        for v in vals {
            match v {
                BusHealth::Connected => {}
                BusHealth::Disconnected => {}
            }
        }
    }

    #[test]
    fn connected_is_not_disconnected() {
        assert_ne!(BusHealth::Connected, BusHealth::Disconnected);
    }

    #[test]
    fn as_u8_discriminant_stable() {
        // Not relying on specific values, just that they differ.
        let c = BusHealth::Connected as u8;
        let d = BusHealth::Disconnected as u8;
        assert_ne!(c, d);
    }

    #[test]
    fn send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BusHealth>();
    }

    #[test]
    fn display_debug_same() {
        let c = BusHealth::Connected;
        let d = BusHealth::Disconnected;
        assert_eq!(format!("{c:?}"), "Connected");
        assert_eq!(format!("{d:?}"), "Disconnected");
    }
}
