load("@vx//stdlib:provider.star", "runtime_def", "bundled_runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend", "env_set")

name = "renderdoc"
description = "RenderDoc - Open-source graphics frame capture and debugging"
homepage = "https://renderdoc.org"
repository = "https://github.com/baldurk/renderdoc"
license = "MIT"
ecosystem = "system"
runtimes = [
    runtime_def("renderdoc", executable = "qrenderdoc", aliases = ["qrenderdoc"], test_commands = [
        # --version initializes QApplication before processing command-line options.
        {"command": "{executable}", "check_type": "check_file", "name": "installed_application_executable"},
    ], platform_constraint = {"os": ["windows", "linux"]}),
    bundled_runtime_def("renderdoccmd", "renderdoc", description = "RenderDoc capture command line", test_commands = [
        {"command": "{executable} version", "name": "version_check", "expected_output": "renderdoccmd"},
    ], platform_constraint = {"os": ["windows", "linux"]}),
]
permissions = github_permissions(extra_hosts = ["renderdoc.org"])
fetch_versions = make_fetch_versions("baldurk", "renderdoc")

def _prefix(ctx, version):
    if ctx.platform.os == "windows" and ctx.platform.arch in ["x64", "x86"]:
        return "RenderDoc_{}_{}".format(version, "64" if ctx.platform.arch == "x64" else "32")
    if ctx.platform.os == "linux" and ctx.platform.arch == "x64":
        return "renderdoc_{}".format(version)
    return None

def download_url(ctx, version):
    prefix = _prefix(ctx, version)
    if prefix == None:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://renderdoc.org/stable/{}/{}.{}".format(version, prefix, ext)

def install_layout(ctx, version):
    prefix = _prefix(ctx, version)
    if prefix == None:
        return None
    paths = ["qrenderdoc.exe", "renderdoccmd.exe"] if ctx.platform.os == "windows" else ["bin/qrenderdoc", "bin/renderdoccmd"]
    return {"type": "archive", "strip_prefix": prefix, "executable_paths": paths, "required_paths": paths}

def store_root(ctx):
    return ctx.vx_home + "/store/renderdoc"

def _executable_path(ctx, executable):
    return ctx.install_dir + ("/" + executable + ".exe" if ctx.platform.os == "windows" else "/bin/" + executable)

def get_execute_path(ctx, version):
    if _prefix(ctx, version) == None:
        return None
    executable = "renderdoccmd" if ctx.runtime_name == "renderdoccmd" else "qrenderdoc"
    return _executable_path(ctx, executable)

def environment(ctx, version):
    if _prefix(ctx, version) == None:
        return []
    return [
        env_prepend("PATH", ctx.install_dir if ctx.platform.os == "windows" else ctx.install_dir + "/bin"),
        env_set("DCC_MCP_RENDERDOC_CMD", _executable_path(ctx, "renderdoccmd")),
    ]
