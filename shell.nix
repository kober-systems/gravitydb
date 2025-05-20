let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { overlays = [ (import sources.rust-overlay) ]; };
  rust = pkgs.rust-bin.stable.latest.minimal;
in
pkgs.mkShell {
  buildInputs = [
    rust
    pkgs.cargo-expand
    pkgs.deno

    # keep this line if you use bash
    pkgs.bashInteractive
  ];
}
