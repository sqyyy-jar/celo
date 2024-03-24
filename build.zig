const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const celo_exe = b.addExecutable(.{
        .name = "celo",
        .root_source_file = .{ .path = "celo/main.zig" },
        .target = target,
        .optimize = optimize,
    });
    const maquina_exe = b.addExecutable(.{
        .name = "maq",
        .root_source_file = .{ .path = "maquina/main.zig" },
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(celo_exe);
    b.installArtifact(maquina_exe);

    const run_cmd = b.addRunArtifact(celo_exe);

    run_cmd.step.dependOn(b.getInstallStep());

    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run_cmd.step);

    const unit_tests = b.addTest(.{
        .root_source_file = .{ .path = "celo/main.zig" },
        .target = target,
        .optimize = optimize,
    });

    const run_unit_tests = b.addRunArtifact(unit_tests);

    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_unit_tests.step);
}
