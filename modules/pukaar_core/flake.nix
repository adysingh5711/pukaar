{
  description = "Pukaar core: a community issue tracker where residents confirm the fix";

  # Pre-built delivery module from the public Logos cache (needs a trusted user, Task 0).
  nixConfig = {
    extra-substituters = [ "https://cache.nix.logos.co/public" ];
    extra-trusted-public-keys = [ "public:l4HrXgL4nw246+LBh2SOJyhz64BoGegOYLheT/iIAPU=" ];
  };

  inputs = {
    logos-module-builder = {
      url = "github:logos-co/logos-module-builder/0.3.1";
      # The wallet crate of LEZ v0.3.1 needs rustc 1.98.1. The overlay of the builder knows up to 1.98.0.
      inputs.rust-overlay.follows = "rust-overlay";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay/b2ccad3b67335d368eef49ec48b9c8166858e73d";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Input name must equal the dependency name. follows = one builder everywhere (basecamp#150).
    delivery_module = {
      url = "github:logos-co/logos-delivery-module/v0.3.0-rc.2";
      inputs.logos-module-builder.follows = "logos-module-builder";
    };
    # Only the typed client wrapper is generated from these two. The .lgx of each is installed into Basecamp on its own.
    storage_module = {
      url = "github:logos-co/logos-storage-module/v3.0.2";
      inputs.logos-module-builder.follows = "logos-module-builder";
    };
    # Private 1:1 chat (chat-module.md). It shares our builder and delivery_module, so the lock stays small.
    chat_module = {
      url = "github:logos-co/logos-chat-module/v0.3.0";
      inputs.logos-module-builder.follows = "logos-module-builder";
      inputs.logos-delivery-module.follows = "delivery_module";
    };
    nixpkgs.follows = "logos-module-builder/nixpkgs";
  };

  outputs = inputs@{ logos-module-builder, nixpkgs, ... }:
    let
      lib = nixpkgs.lib;
      systems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ];
      # The builder copies only rust-lib/ into the crate build, so the path dependency of `logic` on
      # programs/pukaar_sites/rules (outside this folder) cannot resolve. Copy the rules crate from
      # the repo into logic/ at build time and point the path at the copy. Git keeps one copy.
      srcFor = system: nixpkgs.legacyPackages.${system}.runCommand "pukaar_core-src" { } ''
        cp -r ${./.} $out
        chmod -R u+w $out
        cp -r ${../../programs/pukaar_sites/rules} $out/rust-lib/logic/pukaar_sites_rules
        substituteInPlace $out/rust-lib/logic/Cargo.toml \
          --replace-fail '../../../../programs/pukaar_sites/rules' 'pukaar_sites_rules'
      '';
      # Two build scripts of the wallet tree download a file. The Nix sandbox has no network, so Nix
      # fetches both (fixed hash) and the scripts read them from these variables.
      # risc0-circuit-recursion 4.0.5: build.rs checks SHA256_HASH and names the URL by it.
      zkr = pkgs: pkgs.fetchurl {
        url = "https://risc0-artifacts.s3.us-west-2.amazonaws.com/zkr/744b999f0a35b3c86753311c7efb2a0054be21727095cf105af6ee7d3f4d8849.zip";
        sha256 = "744b999f0a35b3c86753311c7efb2a0054be21727095cf105af6ee7d3f4d8849";
      };
      # logos-blockchain-circuits 0.5.7 (the version in Cargo.lock): the release tarball of its own
      # circuits-nix-hashes.json, unpacked with the top folder removed.
      circuits = pkgs:
        let
          os = if pkgs.stdenv.isDarwin then "macos" else "linux";
          arch = if pkgs.stdenv.isAarch64 then "aarch64" else "x86_64";
          hashes = {
            aarch64-darwin = "sha256-daUohb87Oq1RNFf5Kf5IQs1Rs8RiJuFF8hoIWLuLSyU=";
            aarch64-linux = "sha256-Hqo+M52F10HdcDFMStRbfuZ8K6r2xV41FwgBFhw3LXs=";
            x86_64-linux = "sha256-kqDGrYENSGYxUYt7myR6K2b2avb4+N3OOBrLvf7cD38=";
          };
        in pkgs.runCommand "logos-blockchain-circuits-0.5.7" {
          src = pkgs.fetchurl {
            url = "https://github.com/logos-blockchain/logos-blockchain-circuits/releases/download/v0.5.7/logos-blockchain-circuits-v0.5.7-${os}-${arch}.tar.gz";
            sha256 = hashes.${pkgs.stdenv.hostPlatform.system} or (throw "no circuits hash for ${pkgs.stdenv.hostPlatform.system}");
          };
        } "mkdir -p $out && tar -xzf $src -C $out --strip-components=1";
      # rust-rapidsnark (rev e91187f, RAPIDSNARK_VERSION 0.0.8): build.rs runs a download script unless
      # RAPIDSNARK_LIB_DIR names a folder with the static libs. Same files and hashes as its own flake.nix.
      rapidsnark = pkgs:
        let
          sys = pkgs.stdenv.hostPlatform.system;
          fork = "https://github.com/logos-blockchain/logos-blockchain-rust-rapidsnark/releases/download/rapidsnark-pic-v0.0.8";
          iden3 = "https://github.com/iden3/rapidsnark/releases/download/v0.0.8";
          asset = {
            x86_64-linux = { url = "${fork}/rapidsnark-linux-x86_64-pic-v0.0.8.zip"; hash = "sha256-88+TkECQYCKBN0WbYLRB+qi6TEhbjVfrpCqlSgm0DR8="; };
            aarch64-linux = { url = "${fork}/rapidsnark-linux-aarch64-pic-v0.0.8.zip"; hash = "sha256-kcSITwWZjEv9gadzYL7LiG+DVtEEFbQJuhd/LTyOxJU="; };
            aarch64-darwin = { url = "${iden3}/rapidsnark-macOS-arm64-v0.0.8.zip"; hash = "sha256-DmhMmMq7/HKsx+Iz4rdsKYBDEwqwMZALo+ZJePJvAJg="; };
            x86_64-darwin = { url = "${iden3}/rapidsnark-macOS-x86_64-v0.0.8.zip"; hash = "sha256-/GCXzzT5mkBeXkVQAGEF9OmJXXcYz4KoXNzjFvhSgNU="; };
          }.${sys};
        in pkgs.runCommand "rapidsnark-0.0.8" { src = pkgs.fetchzip asset; } "mkdir -p $out && cp $src/lib/* $out/";
      # lee v0.3.1 (build.rs of lee, build_utils): it looks for a folder `artifacts` above its own crate.
      # The Nix vendor folder holds the crate alone, so the build would stop. This setup hook copies the
      # folder from the same git revision into the build directory, above the vendor folder.
      artifactsHook = pkgs:
        let
          repo = builtins.fetchGit {
            url = "https://github.com/logos-blockchain/logos-execution-zone.git";
            rev = "1a003bfbfb89fcf987f62975cd08a401169e1b8c";
            allRefs = true;
          };
        in pkgs.makeSetupHook { name = "lez-artifacts-hook"; } (pkgs.writeText "lez-artifacts-hook.sh" ''
          copyLezArtifacts() { cp -r ${repo}/artifacts "$NIX_BUILD_TOP/artifacts"; }
          postUnpackHooks+=(copyLezArtifacts)
        '');
      forSystem = system:
        let pkgs = nixpkgs.legacyPackages.${system}; in
        logos-module-builder.lib.mkLogosModule {
          src = srcFor system;
          configFile = ./metadata.json;
          flakeInputs = inputs;
          rustExtraNativeBuildInputs = [ (artifactsHook pkgs) ];
          rustEnv = { RECURSION_SRC_PATH = zkr pkgs; LBC_ROOT_DIR = circuits pkgs; RAPIDSNARK_LIB_DIR = rapidsnark pkgs; };
        };
      out = lib.genAttrs systems forSystem;
      # One src per system, so each output is taken from the call that built its own src.
      # Only these three are used by us and by dependants (pukaar_ui reads configFor and packages.*.lgx).
      per = name: lib.genAttrs systems (s: out.${s}.${name}.${s});
    in
    { packages = per "packages"; devShells = per "devShells"; configFor = per "configFor"; };
}
