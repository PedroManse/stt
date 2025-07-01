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
    version = "1.0.0";
    owner = "PedroManse";
    repo = "stck";

    src = ./.;

    cargoHash = "sha256-2Lq78ynU0EMuFS8SVsJ53kidEzEAP7lobc6HUGKAFzw=";

    meta = with lib; {
      description = " Stack based scripting language";
      homepage = "https://github.com/${owner}/${repo}";
      license = with licenses; [ gpl3 ];
      maintainers = with maintainers; [ ];
    };
  }
) { }
