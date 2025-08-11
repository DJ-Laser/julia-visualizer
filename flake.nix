{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
  } @ inputs: let
    system = "x86_64-linux";
    overlays = [(fenix.overlays.default)];
    pkgs = import nixpkgs {inherit system overlays;};
    lib = pkgs.lib;

    wgpuDeps = with pkgs; [
      libxkbcommon
      wayland
      xorg.libX11
      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
      alsa-lib
      fontconfig
      freetype
      shaderc
      directx-shader-compiler
      pkg-config
      cmake
      mold

      libGL
      vulkan-headers
      vulkan-loader
      vulkan-tools
      vulkan-tools-lunarg
      vulkan-extension-layer
      vulkan-validation-layers

      cargo-nextest
      cargo-fuzz

      # nice for developing wgpu itself
      typos

      # if you don't already have rust installed through other means,
      # this shell.nix can do that for you with this below
      yq # for tomlq below
      rustup

      # nice tools
      gdb
      rr
      evcxr
      valgrind
      renderdoc
    ];

    rustToolchain = pkgs.fenix.stable.toolchain;
    packages = with pkgs; [alejandra rustToolchain] ++ wgpuDeps;
  in {
    devShells.${system}.default = pkgs.mkShell {
      inherit packages;
      RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/src";
      shellHook = ''
        export PATH="$PATH:''${CARGO_HOME:-~/.cargo}/bin"
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${lib.makeLibraryPath packages}";
      '';
    };
  };
}
