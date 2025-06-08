{ pkgs ? import <nixpkgs> {} }:
with pkgs; mkShell {
    buildInputs = with pkgs; [
        cmake
        redis
    ];
}
