const compiler = @import("../compiler.zig");

pub const Kind = enum {
    // Literals
    integer,
    float,
    string,
    // Brackets
    left_paren,
    right_paren,
    left_square,
    right_square,
    left_curly,
    right_curly,
    // Identifiers
    identifier,
    bang_identifier,
    dot_identifier,
    // Keywords
    right_arrow,
};

pub const keywords = [_]struct { str: []const u8, kind: Kind }{
    .{ .str = "->", .kind = .right_arrow },
};

location: compiler.Location,
kind: Kind,
