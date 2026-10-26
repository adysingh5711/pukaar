{
  description = "Pukaar core: civic fault reports that can't be quietly closed";

  # Pre-built delivery module from the public Logos cache (needs a trusted user, Task 0).
  nixConfig = {
    extra-substituters = [ "https://cache.nix.logos.co/public" ];
    extra-trusted-public-keys = [ "public:l4HrXgL4nw246+LBh2SOJyhz64BoGegOYLheT/iIAPU=" ];
  };

  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder/0.3.1";
    # Input name must equal the dependency name. follows = one builder everywhere (basecamp#150).
    delivery_module = {
      url = "github:logos-co/logos-delivery-module/v0.3.0-rc.2";
      inputs.logos-module-builder.follows = "logos-module-builder";
    };
    # Only the typed client wrapper is generated from these two. The .lgx of each is installed into Basecamp on its own.
    storage_module = {
      url = "github:logos-co/logos-storage-module/v3.0.0";
      inputs.logos-module-builder.follows = "logos-module-builder";
    };
    # Local branch import-public-account (adds import_public_account), not the stock 0.5.1.
    lez_core = {
      url = "git+file:///Users/aditya/Dev/Hackathons/logos-works/logos-execution-zone-module?ref=import-public-account&rev=3080cfd7ad8a719a48f5e92b462cc9c063b7fc29";
      inputs.logos-module-builder.follows = "logos-module-builder";
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
      forSystem = system: logos-module-builder.lib.mkLogosModule {
        src = srcFor system;
        configFile = ./metadata.json;
        flakeInputs = inputs;
      };
      out = lib.genAttrs systems forSystem;
      # One src per system, so each output is taken from the call that built its own src.
      # Only these three are used by us and by dependants (pukaar_ui reads configFor and packages.*.lgx).
      per = name: lib.genAttrs systems (s: out.${s}.${name}.${s});
    in
    { packages = per "packages"; devShells = per "devShells"; configFor = per "configFor"; };
}
