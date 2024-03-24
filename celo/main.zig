const std = @import("std");
const cli = @import("cli.zig");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    var args = try std.process.argsWithAllocator(gpa.allocator());
    _ = args.skip();
    cli.run(&args, gpa.allocator());
}
