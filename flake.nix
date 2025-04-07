{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  description = "The native part of the Pipewire Screenaudio extension";

  outputs = { self, nixpkgs, }:
    let
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = nixpkgs.legacyPackages;
      read = builtins.readFile;
      systems = [ "aarch64-linux" "i686-linux" "x86_64-linux" ];
      write = builtins.toFile;

      mkDate = longDate:
        with builtins;
        (concatStringsSep "-" [
          (substring 0 4 longDate)
          (substring 4 2 longDate)
          (substring 6 2 longDate)
        ]);

      firefoxJSON = write "firefox.json" (read native/native-messaging-hosts/firefox.json);
      connectorPath = "lib/mozilla/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json";
    in {
      packages = forAllSystems (system: {
        default = with pkgsFor.${system};
          rustPlatform.buildRustPackage {
            name = "pipewire-screenaudio";
            version = mkDate (self.lastModifiedDate or "19700101") + "_"
              + (self.shortRev or "dirty");

            src = ./native/connector-rs;
            buildInputs = [ pipewire ];
            cargoHash = "sha256-H/Uf6Yo8z6tZduXh1zKxiOqFP8hW7Vtqc7p5GM8QDws=";

            postInstall = ''
              # Firefox manifest
              install -Dm644 ${firefoxJSON} "$out/${connectorPath}"
              substituteInPlace "$out/${connectorPath}" --replace "/usr/lib/pipewire-screenaudio/connector-rs/target/debug" "$out/bin"
            '';
          };
      });
    };
}
