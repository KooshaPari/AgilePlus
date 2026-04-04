import Foundation

/// Feature Requirement (FR) traceability for Swift tests.
///
/// Use `traceTo()` to mark a test as tracing to an FR.
///
/// Example:
/// ```swift
/// func testFeature() throws {
///     try traceTo("FR-EXAMPLE-001")
///     // test code
/// }
/// ```
public func traceTo(_ frId: String) throws {
    if !validateFrId(frId) {
        throw TraceabilityError.invalidFrId(frId)
    }
    
    // Store for collection
    TraceCollector.shared.record(frId)
    
    // Log if verbose
    if ProcessInfo.processInfo.environment["VERBOSE"] != nil {
        print("[TRACE] Test traces to: \(frId)")
    }
}

/// Marks a test as tracing to multiple FRs.
public func traceTo(_ frIds: [String]) throws {
    for frId in frIds {
        try traceTo(frId)
    }
}

/// Validates an FR ID format.
public func validateFrId(_ frId: String) -> Bool {
    let pattern = "^FR-[A-Z][A-Z0-9]*-\\d{3,}(-[A-Z0-9]+)?$"
    let regex = try! NSRegularExpression(pattern: pattern, options: [])
    let range = NSRange(location: 0, length: frId.utf16.count)
    return regex.firstMatch(in: frId, options: [], range: range) != nil
}

/// Errors that can occur during traceability operations.
public enum TraceabilityError: Error {
    case invalidFrId(String)
}

/// Collects FR traces during test runs.
public class TraceCollector {
    public static let shared = TraceCollector()
    
    private var traces: [String] = []
    
    private init() {}
    
    public func record(_ frId: String) {
        traces.append(frId)
    }
    
    public func getTraces() -> [String] {
        return Array(Set(traces)).sorted()
    }
    
    public func reset() {
        traces.removeAll()
    }
}
