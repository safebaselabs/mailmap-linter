{rustPlatform}:
rustPlatform.buildRustPackage {
  pname = "mailmap-linter";
  version = "0.1.0";
  src = ../../.;
  cargoLock.lockFile = ../../Cargo.lock;
  doCheck = false;
}
