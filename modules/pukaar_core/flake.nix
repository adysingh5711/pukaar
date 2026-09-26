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
  };

  outputs = inputs@{ logos-module-builder, ... }:
    logos-module-builder.lib.mkLogosModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
    };
}
