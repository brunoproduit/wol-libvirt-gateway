{
  pkgs,
  package,
}:
pkgs.mkShell {
  inputsFrom = [package];
  buildInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
  ];
}
