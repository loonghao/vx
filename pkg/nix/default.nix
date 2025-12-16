{ lib
, stdenv
, fetchFromGitHub
, rustPlatform
, pkg-config
, openssl
, darwin
}:

rustPlatform.buildRustPackage rec {
  pname = "vx";
  version = "{{VERSION}}";

  src = fetchFromGitHub {
    owner = "loonghao";
    repo = "vx";
    rev = "v${version}";
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  };

  cargoHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  # Skip tests that require network access
  checkFlags = [
    "--skip=integration"
  ];

  meta = with lib; {
    description = "Universal version manager for developer tools";
    longDescription = ''
      vx is a fast, cross-platform version manager for developer tools
      including Node.js (npm, pnpm, yarn, bun), Python (uv, pip), Go, Rust,
      and more. It automatically detects and installs the right tool versions
      for your projects.
    '';
    homepage = "https://github.com/loonghao/vx";
    license = licenses.mit;
    maintainers = with maintainers; [ ];
    mainProgram = "vx";
    platforms = platforms.unix;
  };
}
