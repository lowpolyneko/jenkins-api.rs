with import <nixpkgs> { };

mkShell {
  packages = [
    openssl
    pkg-config
    rustup
  ];
}

# vim: ts=2:sw=2:expandtab:
