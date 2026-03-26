{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  description = "The native part of the Pipewire Screenaudio extension";

  outputs =
    { self, nixpkgs }:
    let
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = nixpkgs.legacyPackages;
      read = builtins.readFile;
      systems = [
        "aarch64-linux"
        "i686-linux"
        "x86_64-linux"
      ];
      write = builtins.toFile;

      mkDate =
        longDate:
        with builtins;
        (concatStringsSep "-" [
          (substring 0 4 longDate)
          (substring 4 2 longDate)
          (substring 6 2 longDate)
        ]);

      manifestJSON = write "manifest.json" (read native/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json);
      connectorPath = "lib/mozilla/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json";
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor.${system};
          fs = pkgs.lib.fileset;
        in
        rec {
          default =
            with pkgs;
            rustPlatform.buildRustPackage {
              name = "pipewire-screenaudio";
              version = mkDate (self.lastModifiedDate or "19700101") + "_" + (self.shortRev or "dirty");

              src = ./native/connector-rs;
              nativeBuildInputs = [
                pkg-config
              ];
              buildInputs = [
                pipewire
              ];
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
                libclang
              ];
              BINDGEN_EXTRA_CLANG_ARGS = ''
                -I${glibc.dev}/include
              '';
              cargoLock.lockFile = ./native/connector-rs/Cargo.lock;

              postInstall = ''
                # Firefox manifest
                install -Dm644 ${manifestJSON} "$out/${connectorPath}"
                substituteInPlace "$out/${connectorPath}" \
                    --replace "CONNECTOR_BINARY_PATH" "$out/bin/connector-rs" \
                    --replace "ALLOWED_FIELD" "allowed_extensions" \
                    --replace "ALLOWED_VALUE" "pipewire-screenaudio@icenjim"
              '';
            };
          extension-react = pkgs.stdenvNoCC.mkDerivation (finalAttrs: {
            pname = "pipewire-screenaudio-extension-react";
            version = "0.4.2";

            src = fs.toSource {
              root = ./.;
              fileset = fs.unions [
                ./yarn.lock
                ./package.json
                ./extension/react
              ];
            };

            yarnOfflineCache = pkgs.fetchYarnDeps {
              yarnLock = finalAttrs.src + "/yarn.lock";
              hash = "sha256-5Wv5Rup62XVFDgmH/Bgb9C6JdMqHApYEs8YVwfTgkgg=";
            };

            nativeBuildInputs = with pkgs; [
              yarnConfigHook
              yarnBuildHook
              # Needed for executing package.json scripts
              nodejs
            ];
            installPhase = ''
              cp -r extension/react/dist $out/
            '';
          });
          extension =
            pkgs.runCommand "pipewire-screenaudio-extension"
              {
                src = fs.toSource {
                  root = ./extension;
                  fileset = fs.difference ./extension (
                    fs.fileFilter (file: file.name == ".prettierrc.yml") ./extension
                  );
                };
                nativeBuildInputs = with pkgs; [
                  zip
                ];
              }
              ''
                mkdir -p release

                mkdir -p release/react
                ln -s ${extension-react} release/react/dist

                cp -r $src/scripts $src/assets $src/manifest.json release/

                cd release
                zip -r - . > $out
              '';
        }
      );
      devShells = forAllSystems (
        system: with pkgsFor.${system}; {
          default = mkShell {
            buildInputs = [
              cargo
              clippy
              nodejs
              pipewire
              pkg-config
              rust-analyzer
              rustc
              rustfmt
              yarn
            ];
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
              libclang
            ];
            BINDGEN_EXTRA_CLANG_ARGS = ''
              -I${glibc.dev}/include
            '';
          };
        }
      );
    };
}
