const std = @import("std");
const fs = std.fs;
const mem = std.mem;

const Self = @This();
const max_source_size = 1024 * 1024 * 16; // 16 MiB

path: []const u8,
content: []const u8,
allocator: mem.Allocator,

pub fn load(path: []const u8, allocator: mem.Allocator) !Self {
    const file = try fs.cwd().openFile(path, .{});
    var out = std.ArrayList(u8).init(allocator);
    errdefer out.deinit();
    try file.reader().readAllArrayList(&out, max_source_size);
    const my_path = try allocator.alloc(u8, path.len);
    @memcpy(my_path, path);
    return Self{
        .path = my_path,
        .content = try out.toOwnedSlice(),
        .allocator = allocator,
    };
}

pub fn deinit(self: Self) void {
    self.allocator.free(self.path);
    self.allocator.free(self.content);
}
