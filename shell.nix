(import (
  let lock = builtins.fromJSON (builtins.readFile ./flake.lock);
    nodeName = lock.nodes.root.inputs.flake-compat;
  in
  fetchTarball {
    url = lock.nodes.${nodeName}.locked.url or "https://git.lix.systems/lix-project/flake-compat/archive/master.tar.gz";
    sha256 = lock.nodes.${nodeName}.locked.narHash;
  }
) { src = ./.; }).shellNix
