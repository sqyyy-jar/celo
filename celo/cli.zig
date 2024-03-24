const std = @import("std");
const debug = std.debug;
const mem = std.mem;
const process = std.process;
const compiler = @import("compiler.zig");

const Subcommand = struct {
    name: []const u8,
    alias: []const u8,
    description: []const u8 = "",
    handler: *const fn (*process.ArgIterator, mem.Allocator) void,
};

const subcommands = [_]Subcommand{
    .{ .name = "help", .alias = "h", .description = "Display this help and exit.", .handler = commandHelp },
    .{ .name = "run", .alias = "r", .description = "Run a file.", .handler = commandRun },
};

/// This function will never return.
pub fn run(args: *process.ArgIterator, allocator: mem.Allocator) noreturn {
    const arg = args.next() orelse {
        printHelp();
        process.exit(1);
    };
    for (&subcommands) |*subcommand| {
        if (mem.eql(u8, arg, subcommand.name) or mem.eql(u8, arg, subcommand.alias)) {
            subcommand.handler(args, allocator);
            process.exit(0);
        }
    }
    printHelp();
    process.exit(1);
}

fn printHelp() void {
    debug.print(
        \\The Celo programming language
        \\
        \\Commands:
        \\
    , .{});
    const alignment = 6;
    const spaces = " " ** alignment;
    for (&subcommands) |*subcommand| {
        const offset = alignment -| subcommand.name.len;
        debug.print(
            \\    {s}, {s}
        , .{ subcommand.name, subcommand.alias });
        if (subcommand.description.len > 0) {
            debug.print("{s} {s}", .{ spaces[0..offset], subcommand.description });
        }
        debug.print("\n", .{});
    }
}

fn commandHelp(args: *process.ArgIterator, _: mem.Allocator) void {
    printHelp();
    if (args.skip()) {
        process.exit(1);
    }
}

fn commandRun(args: *process.ArgIterator, allocator: mem.Allocator) void {
    var arg_file_path: ?[]const u8 = null;
    while (args.next()) |arg| {
        if (!mem.startsWith(u8, arg, "-")) {
            arg_file_path = arg;
            break;
        }
        debug.print("Flags are not implemented yet.\n", .{});
        process.exit(1);
        const flag = arg[1..];
        if (mem.startsWith(u8, flag, "-")) {
            const long_flag = flag[1..];
            _ = long_flag;
        }
    }
    const file_path = arg_file_path orelse {
        debug.print("No file was provided.\n", .{});
        process.exit(1);
    };
    var cc = compiler.Compiler.init(allocator);
    defer cc.deinit();
    cc.addSource(file_path) catch |err| {
        debug.print("Could not add source: {}\n", .{err});
        process.exit(1);
    };
    cc.compile();
}
