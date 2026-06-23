{
  pkgs ? import <nixpkgs> { },
}:
with pkgs;
let
  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
in
callPackage (
  {
    stdenv,
    mkShell,
    rustup,
    rustPlatform,
  }:
  mkShell {
    strictDeps = true;
    nativeBuildInputs = [
      rustup
      rustPlatform.bindgenHook
      pkg-config
    ];
    # libraries here
    buildInputs =
      [
        webkitgtk_4_1
        gtk3
        glib
        gobject-introspection
        pango
        atk
        cairo
        gdk-pixbuf
        openssl
        libsoup_3
        xdotool
      ];
    RUSTC_VERSION = overrides.toolchain.channel;
    # https://github.com/rust-lang/rust-bindgen#environment-variables
    shellHook = ''
      export PATH="''${CARGO_HOME:-~/.cargo}/bin":"$PATH"
      export PATH="''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-${stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
    '';

    packages = [
      dioxus-cli
      tailwindcss_4
      wasm-bindgen-cli
    ];
  }
) { }
