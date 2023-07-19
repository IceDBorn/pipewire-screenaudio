{ lib
, stdenv
, fetchFromGitHub
, gawk
, hexdump
, jq
, pipewire
, psmisc
,
}:
stdenv.mkDerivation rec {
  pname = "pipewire-screenaudio";
  version = "0.2.0";

  src = fetchFromGitHub {
    owner = "IceDBorn";
    repo = "pipewire-screenaudio";
    rev = version;
    hash = "sha256-IfPW0qmIUMIuevMLolYyKpYMBiiBG1OJA7/Wtxp+EzM=";
  };

  buildInputs = [
    gawk
    hexdump
    jq
    pipewire
    psmisc
  ];

  installPhase = ''
    runHook preInstall

    mkdir -p $out/lib/out
    install -Dm755 native/connector/pipewire-screen-audio-connector.sh $out/lib/connector/pipewire-screen-audio-connector.sh
    install -Dm755 native/connector/virtmic.sh $out/lib/connector/virtmic.sh

    # Firefox manifest
    sed -i "s|/usr/lib/pipewire-screenaudio|$out/lib|g" native/native-messaging-hosts/firefox.json
    install -Dm644 native/native-messaging-hosts/firefox.json $out/lib/mozilla/native-messaging-hosts/com.icedborn.pipewirescreenaudioconnector.json

    runHook postInstall
  '';

  meta = with lib; {
    description = "Extension to passthrough pipewire audio to WebRTC Screenshare";
    homepage = "https://github.com/IceDBorn/pipewire-screenaudio";
    license = licenses.gpl3Only;
    maintainers = with maintainers; [ icedborn ];
    platforms = lib.platforms.linux;
  };
}
