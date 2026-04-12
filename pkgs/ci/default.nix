{pkgs}:
pkgs.writeShellApplication {
  name = "ci";
  runtimeInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    cargo-tarpaulin
    git
    gcc
    pkg-config
  ];
  text = ''
    echo "Running cargo fmt..."
    cargo fmt

    echo "Running cargo clippy..."
    cargo clippy

    echo "Running tests with coverage..."
    cargo tarpaulin --out Stdout --timeout 300

    echo "All CI checks passed!"
  '';
}
