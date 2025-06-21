let
  pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-25.05") { };
in
pkgs.callPackage (
  {
    lib,
    rustPlatform,
  }:
  rustPlatform.buildRustPackage rec {
    pname = "stck";
    version = "0.3.0";
    owner = "PedroManse";
    repo = "stck";

    src = ./.;

    cargoHash = "sha256-iJF6LA5SJ3wKJbrnpMVIIbw01maTy9nE//dgxFFoAOk=";

    meta = with lib; {
      description = " Stack based scripting language";
      homepage = "https://github.com/${owner}/${repo}";
      license = with licenses; [ gpl3 ];
      maintainers = with maintainers; [ ];
    };
  }
) { }
