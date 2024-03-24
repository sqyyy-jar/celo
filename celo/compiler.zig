pub const hir = @import("compiler/hir.zig");

pub const Compiler = @import("compiler/Compiler.zig");
pub const Lexer = @import("compiler/Lexer.zig");
pub const Parser = @import("compiler/Parser.zig");

pub const Source = @import("compiler/Source.zig");
pub const Token = @import("compiler/Token.zig");
pub const Location = @import("compiler/Location.zig");

const result = @import("result.zig");

pub const Error = union(enum) {
    lexer_error: Lexer.Error,
    parser_error: Parser.Error,
};

pub fn Result(comptime T: type) type {
    return result.Result(T, Error);
}
