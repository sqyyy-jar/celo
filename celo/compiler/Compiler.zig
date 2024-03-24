const std = @import("std");
const compiler = @import("../compiler.zig");
const mem = std.mem;

const Self = @This();

sources: std.ArrayList(compiler.Source),
allocator: mem.Allocator,

pub fn init(allocator: mem.Allocator) Self {
    return Self{
        .sources = std.ArrayList(compiler.Source).init(allocator),
        .allocator = allocator,
    };
}

pub fn addSource(self: *Self, path: []const u8) !void {
    const source = try compiler.Source.load(path, self.allocator);
    try self.sources.append(source);
}

pub fn compile(self: *Self) void {
    for (self.sources.items) |*source| {
        var parser = compiler.Parser.init(self, source);
        const scope = parser.parseScope(null);
        std.debug.print("{any}\n", .{scope});
    }
}

pub fn deinit(self: Self) void {
    self.sources.deinit();
}
