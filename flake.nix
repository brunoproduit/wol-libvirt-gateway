{
  description = "NixOS Wake-on-LAN gateway for libvirt VMs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      packages.default = pkgs.callPackage ./packaging/nix/package.nix {};
      devShells.default = pkgs.callPackage ./packaging/nix/shell.nix {
        inherit pkgs;
        package = self.packages.${system}.default;
      };
    })
    // {
      overlays.default = import ./packaging/nix/overlay.nix;
      nixosModules.default = import ./packaging/nix/module.nix;
    };
}
