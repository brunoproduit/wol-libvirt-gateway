{
  lib,
  rustPlatform,
  pkg-config,
  libcap,
  libvirt,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../../Cargo.toml);
  inherit (cargoToml.package) name version description repository;
in
  rustPlatform.buildRustPackage {
    pname = name;
    inherit version;

    src = lib.cleanSource ../../.;

    cargoLock.lockFile = ../../Cargo.lock;

    nativeBuildInputs = [pkg-config];
    buildInputs = [libcap libvirt];

    PKG_CONFIG_PATH = "${lib.getDev libvirt}/lib/pkgconfig";

    meta = {
      inherit description;
      homepage = repository;
    };
  }
