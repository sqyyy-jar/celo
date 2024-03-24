const std = @import("std");
const mem = std.mem;

const Self = @This();

parent: ?usize,
children: std.StringHashMap(usize),
code: std.ArrayList(void),

pub fn init(parent: ?usize, allocator: mem.Allocator) Self {
    return Self{
        .parent = parent,
        .children = std.StringHashMap(usize).init(allocator),
        .code = std.ArrayList(void).init(allocator),
    };
}

pub fn deinit(self: Self) void {
    self.children.deinit();
}
