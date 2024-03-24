const std = @import("std");
const ascii = std.ascii;
const mem = std.mem;
const compiler = @import("../compiler.zig");

const Self = @This();

pub const Error = struct {
    pub const Kind = enum {
        invalid_character,
        invalid_eof,
        invalid_escape_sequence,
    };

    source: []const u8,
    location: compiler.Location,
    kind: Kind,
};

source: []const u8,
start: u24,
start_line: u24,
start_column: u24,
current: u24,
current_line: u24,
current_column: u24,
peek_buf: ?compiler.Token,

pub fn init(source: []const u8) Self {
    return Self{
        .source = source,
        .start = 0,
        .start_line = 1,
        .start_column = 1,
        .current = 0,
        .current_line = 1,
        .current_column = 1,
        .peek_buf = null,
    };
}

fn peek(self: *Self) ?u8 {
    if (self.current >= self.source.len) {
        return null;
    }
    return self.source[self.current];
}

fn next(self: *Self) void {
    if (self.current >= self.source.len) {
        return;
    }
    const c = self.peek();
    self.current += 1;
    self.current_column += 1;
    if (c == '\n') {
        self.current_line += 1;
        self.current_column = 1;
    }
}

fn clearLocation(self: *Self) void {
    self.start = self.current;
    self.start_line = self.current_line;
    self.start_column = self.current_column;
}

fn makeLocation(self: *Self) compiler.Location {
    const location = compiler.Location{
        .start = self.start,
        .end = self.current,
        .line = self.start_line,
        .column = self.start_column,
    };
    self.clearLocation();
    return location;
}

fn makeToken(self: *Self, kind: compiler.Token.Kind) compiler.Token {
    return compiler.Token{
        .location = self.makeLocation(),
        .kind = kind,
    };
}

fn makeError(self: *Self, kind: Error.Kind) compiler.Error {
    return compiler.Error{
        .lexer_error = Error{
            .source = self.source,
            .location = self.makeLocation(),
            .kind = kind,
        },
    };
}

pub fn peekToken(self: *Self) compiler.Result(?compiler.Token) {
    if (self.peek_buf) |token| {
        return .{ .ok = token };
    }
    switch (self.parseToken()) {
        .ok => |token| {
            self.peek_buf = token;
            return .{ .ok = token };
        },
        .err => |err| return .{ .err = err },
    }
}

pub fn consumeToken(self: *Self) void {
    self.peek_buf = null;
}

fn parseToken(self: *Self) compiler.Result(?compiler.Token) {
    while (true) {
        const c = self.peek() orelse {
            return .{ .ok = null };
        };
        self.next();
        if (ascii.isWhitespace(c)) {
            self.clearLocation();
            continue;
        }
        switch (c) {
            ';' => self.skipComment(),
            '(' => return .{ .ok = self.makeToken(.left_paren) },
            ')' => return .{ .ok = self.makeToken(.right_paren) },
            '[' => return .{ .ok = self.makeToken(.left_square) },
            ']' => return .{ .ok = self.makeToken(.right_square) },
            '{' => return .{ .ok = self.makeToken(.left_curly) },
            '}' => return .{ .ok = self.makeToken(.right_curly) },
            '-' => return self.parseMinusPrefix().mapOptional(),
            '.' => return self.parseIdentifier(true).mapOptional(),
            '"' => return self.parseString().mapOptional(),
            '0'...'9' => return self.parseNumber().mapOptional(),
            else => {
                if (isIdentifier(c, true)) {
                    return self.parseIdentifier(false).mapOptional();
                }
                return .{ .err = self.makeError(.invalid_character) };
            },
        }
    }
}

fn skipComment(self: *Self) void {
    while (self.peek()) |c| {
        self.next();
        if (c == '\n') {
            break;
        }
    }
}

fn isIdentifier(c: u8, first: bool) bool {
    if (ascii.isAlphabetic(c)) {
        return true;
    }
    if (!first and ascii.isDigit(c)) {
        return true;
    }
    if (mem.indexOfScalar(u8, "+-*/%=<>&|^_:?%#@~", c)) |_| {
        return true;
    }
    return false;
}

fn parseMinusPrefix(self: *Self) compiler.Result(compiler.Token) {
    const c = self.peek() orelse {
        return self.parseIdentifier(false);
    };
    if (ascii.isDigit(c)) {
        return self.parseNumber();
    }
    return self.parseIdentifier(false);
}

fn parseIdentifier(self: *Self, dot: bool) compiler.Result(compiler.Token) {
    var first = dot;
    var bang = false;
    while (self.peek()) |c| {
        if (c == '.') {
            self.next();
            return .{ .err = self.makeError(.invalid_character) };
        }
        if (bang) {
            if (c == '!' or isIdentifier(c, false)) {
                self.next();
                return .{ .err = self.makeError(.invalid_character) };
            }
            break;
        }
        if (isIdentifier(c, first)) {
            self.next();
            first = false;
            continue;
        }
        if (c == '!') {
            self.next();
            if (dot) {
                return .{ .err = self.makeError(.invalid_character) };
            }
            bang = true;
            continue;
        }
        break;
    }
    if (dot) {
        return .{ .ok = self.makeToken(.dot_identifier) };
    }
    if (bang) {
        return .{ .ok = self.makeToken(.bang_identifier) };
    }
    var token = self.makeToken(.identifier);
    const str = self.source[token.location.start..token.location.end];
    for (&compiler.Token.keywords) |*keyword| {
        if (mem.eql(u8, str, keyword.str)) {
            token.kind = keyword.kind;
            break;
        }
    }
    return .{ .ok = token };
}

fn parseString(self: *Self) compiler.Result(compiler.Token) {
    while (true) {
        const c = self.peek() orelse {
            return .{ .err = self.makeError(.invalid_eof) };
        };
        self.next();
        if (c == '\\') {
            const escape = self.peek() orelse {
                return .{ .err = self.makeError(.invalid_eof) };
            };
            self.next();
            switch (escape) {
                '"' | '\\' | 'n' | 'r' | 't' => {},
                else => {
                    return .{ .err = self.makeError(.invalid_escape_sequence) };
                },
            }
            continue;
        }
        if (c == '"') {
            break;
        }
    }
    return .{ .ok = self.makeToken(.string) };
}

fn parseNumber(self: *Self) compiler.Result(compiler.Token) {
    var dot = false;
    while (self.peek()) |c| {
        if (ascii.isDigit(c)) {
            self.next();
            continue;
        }
        if (c == '.') {
            self.next();
            if (dot) {
                return .{ .err = self.makeError(.invalid_character) };
            }
            dot = true;
            continue;
        }
        break;
    }
    if (dot) {
        return .{ .ok = self.makeToken(.float) };
    }
    return .{ .ok = self.makeToken(.integer) };
}
