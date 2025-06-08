let
  pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-25.05") { };
in
pkgs.callPackage (
  {
    lib,
    rustPlatform,
  }:
  rustPlatform.buildRustPackage rec {
    pname = "stt";
    version = "0.1.0";
    owner = "PedroManse";
    repo = "stt";

    src = ./.;

    cargoHash = "sha256-cLsapNbTkmsxbVag5vufMlpssrHYQ4RAhQT1vLdaf7s=";

    meta = with lib; {
      description = " Stack based scripting language";
      homepage = "https://github.com/${owner}/${repo}";
      license = with licenses; [ gpl3 ];
      maintainers = with maintainers; [ ];
    };
  }
) { }
