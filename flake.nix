{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  description = "The native part of the Pipewire Screenaudio extension";

  outputs = { self, nixpkgs, }:
    let
      pkgsFor = nixpkgs.legacyPackages;
      systems = [ "aarch64-linux" "i686-linux" "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      mkDate = longDate:
        with builtins;
        (concatStringsSep "-" [
          (substring 0 4 longDate)
          (substring 4 2 longDate)
          (substring 6 2 longDate)
        ]);
    in {
      packages = forAllSystems (system: {
        default = with pkgsFor.${system};
          stdenv.mkDerivation {
            name = "pipewire-screenaudio";
            version = mkDate (self.lastModifiedDate or "19700101") + "_"
              + (self.shortRev or "dirty");

            src = self;

            buildInputs = [ gawk hexdump jq pipewire ];

            installPhase = ''
              runHook preInstall

              # Replace jq with its absolute path
              substituteInPlace native/connector/virtmic.sh --replace jq ${pkgs.jq}/bin/jq
              substituteInPlace native/connector/pipewire-screen-audio-connector.sh --replace jq ${pkgs.jq}/bin/jq
              substituteInPlace native/connector/connect-and-monitor.sh --replace jq ${pkgs.jq}/bin/jq
              substituteInPlace native/connector/util.sh --replace jq ${pkgs.jq}/bin/jq

              # Install files
              mkdir -p $out/lib/out
              install -Dm755 native/connector/pipewire-screen-audio-connector.sh $out/lib/connector/pipewire-screen-audio-connector.sh
              install -Dm755 native/connector/virtmic.sh $out/lib/connector/virtmic.sh
              install -Dm755 native/connector/connect-and-monitor.sh $out/lib/connector/connect-and-monitor.sh
              install -Dm755 native/connector/util.sh $out/lib/connector/util.sh

              # Firefox manifest
              substituteInPlace native/native-messaging-hosts/firefox.json --replace /usr/lib/pipewire-screenaudio $out/lib
              install -Dm644 native/native-messaging-hosts/firefox.json $out/lib/mozilla/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json

              runHook postInstall
            '';
          };
      });
    };
}
