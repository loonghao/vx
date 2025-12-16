Name:           vx
Version:        {{VERSION}}
Release:        1%{?dist}
Summary:        Universal version manager for developer tools

License:        MIT
URL:            https://github.com/loonghao/vx
Source0:        https://github.com/loonghao/vx/releases/download/v%{version}/vx-%{_target_cpu}-unknown-linux-gnu.tar.gz

BuildArch:      x86_64 aarch64

%description
vx is a fast, cross-platform version manager for developer tools
including Node.js (npm, pnpm, yarn, bun), Python (uv, pip), Go, Rust,
and more. It automatically detects and installs the right tool versions
for your projects.

Features:
- Automatic version detection from project files
- Multi-ecosystem support (Node.js, Python, Go, Rust)
- Fast parallel downloads
- Cross-platform (Windows, macOS, Linux)
- Zero configuration required

%prep
%setup -q -c

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/licenses/%{name}
mkdir -p %{buildroot}%{_datadir}/doc/%{name}
mkdir -p %{buildroot}%{_datadir}/bash-completion/completions
mkdir -p %{buildroot}%{_datadir}/zsh/site-functions
mkdir -p %{buildroot}%{_datadir}/fish/vendor_completions.d

install -m 755 vx %{buildroot}%{_bindir}/vx
install -m 644 LICENSE %{buildroot}%{_datadir}/licenses/%{name}/LICENSE
install -m 644 README.md %{buildroot}%{_datadir}/doc/%{name}/README.md

# Generate shell completions
%{buildroot}%{_bindir}/vx completions bash > %{buildroot}%{_datadir}/bash-completion/completions/vx 2>/dev/null || true
%{buildroot}%{_bindir}/vx completions zsh > %{buildroot}%{_datadir}/zsh/site-functions/_vx 2>/dev/null || true
%{buildroot}%{_bindir}/vx completions fish > %{buildroot}%{_datadir}/fish/vendor_completions.d/vx.fish 2>/dev/null || true

%files
%license LICENSE
%doc README.md
%{_bindir}/vx
%{_datadir}/bash-completion/completions/vx
%{_datadir}/zsh/site-functions/_vx
%{_datadir}/fish/vendor_completions.d/vx.fish

%changelog
* %(date "+%a %b %d %Y") Long Hao <hal.long@outlook.com> - %{version}-1
- Release v%{version}
