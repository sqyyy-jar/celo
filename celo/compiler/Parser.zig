const std = @import("std");
const mem = std.mem;
const compiler = @import("../compiler.zig");
const hir = compiler.hir;

const Self = @This();

pub const Error = struct {
    pub const Kind = union(enum) {
        unclosed_scope,
        unexpected_token: struct {
            expected: compiler.Token.Kind,
            got: compiler.Token.Kind,
        },
        unexpected_eof: struct {
            expected: ?compiler.Token.Kind,
        },
    };

    source: []const u8,
    location: ?compiler.Location,
    kind: Kind,
};

compiler: *compiler.Compiler,
lexer: compiler.Lexer,

pub fn init(cc: *compiler.Compiler, source: *compiler.Source) Self {
    const lexer = compiler.Lexer.init(source.content);
    return Self{
        .compiler = cc,
        .lexer = lexer,
    };
}

fn makeError(self: *Self, location: ?compiler.Location, kind: Error.Kind) compiler.Error {
    return compiler.Error{
        .parser_error = Error{
            .source = self.lexer.source,
            .location = location,
            .kind = kind,
        },
    };
}

pub fn expectToken(self: *Self, kind: compiler.Token.Kind) compiler.Result(compiler.Token) {
    switch (self.lexer.peekToken()) {
        .ok => |maybe_token| {
            if (maybe_token == null) {
                return .{ .err = self.makeError(null, .{
                    .unexpected_eof = .{ .expected = kind },
                }) };
            }
            const token = maybe_token.?;
            if (token.kind != kind) {
                return .{ .err = self.makeError(token.location, .{
                    .unexpected_token = .{ .expected = kind, .got = token.kind },
                }) };
            }
            return .{ .ok = token };
        },
        .err => |err| return .{ .err = err },
    }
}

pub fn parseScope(self: *Self, parent: ?usize) compiler.Result(hir.Scope) {
    const opening_bracket = if (parent != null) switch (self.expectToken(.left_curly)) {
        .ok => |token| token.location,
        .err => |err| return .{ .err = err },
    } else null;
    var scope = hir.Scope.init(parent, self.compiler.allocator);
    while (true) {
        const token = switch (self.lexer.peekToken()) {
            .ok => |token| token,
            .err => |err| return .{ .err = err },
        } orelse {
            if (opening_bracket) |bracket| {
                return .{ .err = self.makeError(bracket, .unclosed_scope) };
            }
            break;
        };
        self.lexer.consumeToken();
        std.debug.print("token {any}:{d}:{d}\n", .{
            token.kind,
            token.location.line,
            token.location.column,
        });
    }
    return .{ .ok = scope };
}
