# OpenUSD's native Python bindings are distributed as usd-core wheels.
# There is no upstream executable named openusd: this runtime runs Python
# inside uv's isolated usd-core environment, including the native pxr modules.
load("@vx//stdlib:provider.star", "runtime_def", "dep_def", "system_permissions")
load("@vx//stdlib:http.star", "fetch_json_versions")

name = "openusd"
description = "OpenUSD - Python environment with native Universal Scene Description bindings"
homepage = "https://openusd.org"
repository = "https://github.com/PixarAnimationStudios/OpenUSD"
license = "LicenseRef-Tomorrow-Open-Source-Technology-1.0"
ecosystem = "python"
package_alias = {"ecosystem": "uvx", "package": "usd-core", "executable": "python"}

runtimes = [runtime_def("openusd", aliases = ["usd-python"], test_commands = [
    {"command": "{executable} -c \"from pxr import Usd; print(Usd.GetVersion())\"", "name": "native_bindings"},
])]
permissions = system_permissions(extra_hosts = ["pypi.org", "files.pythonhosted.org"], exec_cmds = ["uv", "uvx"])

def fetch_versions(ctx):
    return fetch_json_versions(ctx, "https://pypi.org/pypi/usd-core/json", "pypi")

def download_url(_ctx, _version):
    return None

def store_root(ctx):
    return ctx.vx_home + "/store/openusd"

def get_execute_path(_ctx, _version):
    return None

def environment(_ctx, _version):
    return []

def deps(_ctx, _version):
    return [dep_def("uv", reason = "OpenUSD uses an isolated usd-core Python environment")]
