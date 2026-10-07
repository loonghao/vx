# The official Windows portable distribution includes Python and PyTorch.
# Select a backend explicitly with comfyui-amd / comfyui-intel; comfyui uses
# the upstream NVIDIA package (also usable with the user's --cpu argument).
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:install.star", "create_shim")
load("@vx//stdlib:env.star", "env_prepend")

name = "comfyui"
description = "ComfyUI - Portable node-based generative AI application"
homepage = "https://www.comfy.org"
repository = "https://github.com/Comfy-Org/ComfyUI"
license = "GPL-3.0-or-later"
ecosystem = "ai"
platforms = {"os": ["windows"], "arch": ["x64"]}
runtimes = [
    runtime_def("comfyui", platform_constraint = {"os": ["windows"]}, test_commands = [
        {"command": "{executable} --help", "name": "cli_help", "expected_output": "listen"},
    ]),
    runtime_def("comfyui-amd", platform_constraint = {"os": ["windows"]}, test_commands = [
        {"command": "{executable} --help", "name": "cli_help", "expected_output": "listen"},
    ]),
    runtime_def("comfyui-intel", platform_constraint = {"os": ["windows"]}, test_commands = [
        {"command": "{executable} --help", "name": "cli_help", "expected_output": "listen"},
    ]),
]
permissions = github_permissions()
fetch_versions = make_fetch_versions("Comfy-Org", "ComfyUI", include_prereleases = False)

def _runtime(ctx):
    return ctx.runtime_name if ctx.runtime_name else "comfyui"

def download_url(ctx, version):
    if ctx.platform.os != "windows" or ctx.platform.arch != "x64":
        return None
    backend = {"comfyui": "nvidia", "comfyui-amd": "amd", "comfyui-intel": "intel"}.get(_runtime(ctx))
    if backend == None:
        return None
    return github_asset_url("Comfy-Org", "ComfyUI", "v" + version, "ComfyUI_windows_portable_{}.7z".format(backend))

def install_layout(ctx, _version):
    if ctx.platform.os != "windows" or ctx.platform.arch != "x64":
        return None
    return {"type": "archive", "strip_prefix": "ComfyUI_windows_portable", "executable_paths": ["python_embeded/python.exe"], "required_paths": ["ComfyUI/main.py", "comfyui.cmd"]}

def store_root(ctx):
    return ctx.vx_home + "/store/" + _runtime(ctx)

def get_execute_path(ctx, _version):
    return ctx.install_dir + "/comfyui.cmd"

def post_extract(_ctx, _version, install_dir):
    return [create_shim("comfyui", install_dir + "/python_embeded/python.exe", args = [
        "-s", install_dir + "/ComfyUI/main.py", "--windows-standalone-build",
    ], shim_dir = install_dir)]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
