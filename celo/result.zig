const compiler = @import("compiler.zig");

pub fn Result(comptime T: type, comptime E: type) type {
    return union(enum) {
        const Self = @This();

        /// There was no error.
        ok: T,
        /// There was an error.
        err: E,

        pub fn mapOptional(self: Self) Result(?T, E) {
            return switch (self) {
                .ok => |value| Result(?T, E){ .ok = value },
                .err => |err| Result(?T, E){ .err = err },
            };
        }
    };
}
