//! Zig Traceability Library (zigtreqt)
//!
//! Provides traceability support for Zig tests.

const std = @import("std");

/// Validates an FR ID format.
/// Format: FR-XXXX-NNN or FR-XXXX-NNN-YYY
pub fn validateFrId(fr_id: []const u8) bool {
    // Must start with "FR-"
    if (!std.mem.startsWith(u8, fr_id, "FR-")) return false;
    
    var parts = std.mem.split(u8, fr_id, "-");
    
    // Part 0: "FR"
    _ = parts.next() orelse return false;
    
    // Part 1: Project code (uppercase letters/digits)
    const project = parts.next() orelse return false;
    if (project.len == 0) return false;
    for (project) |c| {
        if (!std.ascii.isUpper(c) and !std.ascii.isDigit(c)) return false;
    }
    
    // Part 2: Number (at least 3 digits)
    const number = parts.next() orelse return false;
    if (number.len < 3) return false;
    for (number) |c| {
        if (!std.ascii.isDigit(c)) return false;
    }
    
    return true;
}

/// Records a trace entry.
pub fn traceTo(allocator: std.mem.Allocator, test_name: []const u8, fr_id: []const u8) !void {
    if (!validateFrId(fr_id)) {
        return error.InvalidFrId;
    }
    
    // Store trace
    try TraceCollector.record(allocator, test_name, fr_id);
    
    // Log if verbose
    if (std.process.getEnvVarOwned(allocator, "VERBOSE")) |_| {
        std.debug.print("[TRACE] {s} -> {s}\n", .{ test_name, fr_id });
    } else |_| {}
}

/// Trace collector for test runs.
pub const TraceCollector = struct {
    var traces: std.ArrayList(TraceEntry) = undefined;
    var initialized = false;
    var mutex = std.Thread.Mutex{};
    
    const TraceEntry = struct {
        test_name: []const u8,
        fr_id: []const u8,
    };
    
    pub fn init(allocator: std.mem.Allocator) void {
        mutex.lock();
        defer mutex.unlock();
        
        if (!initialized) {
            traces = std.ArrayList(TraceEntry).init(allocator);
            initialized = true;
        }
    }
    
    pub fn record(allocator: std.mem.Allocator, test_name: []const u8, fr_id: []const u8) !void {
        init(allocator);
        
        mutex.lock();
        defer mutex.unlock();
        
        try traces.append(.{
            .test_name = try allocator.dupe(u8, test_name),
            .fr_id = try allocator.dupe(u8, fr_id),
        });
    }
    
    pub fn getTraces() []const TraceEntry {
        mutex.lock();
        defer mutex.unlock();
        
        return traces.items;
    }
    
    pub fn reset(allocator: std.mem.Allocator) void {
        mutex.lock();
        defer mutex.unlock();
        
        for (traces.items) |entry| {
            allocator.free(entry.test_name);
            allocator.free(entry.fr_id);
        }
        traces.clearRetainingCapacity();
    }
};
