{
  description = "www.saffi.dev, www.saffi.wtf and their common code";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";

    cargo2nix.url = "github:cargo2nix/cargo2nix/unstable";
    cargo2nix.inputs.flake-utils.follows = "flake-utils";
    cargo2nix.inputs.nixpkgs.follows = "nixpkgs";

    onehalf.url = "github:sonph/onehalf/master";
    onehalf.flake = false;
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , onehalf
    , ...
    } @ inputs:
    let
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [
          inputs.cargo2nix.overlays.default
          inputs.fenix.overlays.default

          (final: prev: {
            cargo2nix = inputs.cargo2nix.packages.${system}.default;

            rust-toolchain =
              let
                inherit (final.lib.strings) fileContents;

                stableFor = target: target.fromToolchainFile {
                  file = ./rust-toolchain.toml;
                  sha256 = "sha256-7QfkHty6hSrgNM0fspycYkRcB82eEqYa4CoAJ9qA3tU=";
                };

                rustfmt = final.fenix.latest.rustfmt;
              in
              final.fenix.combine [
                rustfmt
                (stableFor final.fenix)
              ];

            iosevka = (prev.iosevka.override {
              set = "www-saffi";

              privateBuildPlan = ''
                [buildPlans.iosevka-www-saffi]
                family = "Iosevka www.saffi"
                spacing = "normal"
                serifs = "sans"
                noCvSs = true
                exportGlyphNames = false
                webfontFormats = ["woff2"]

                [buildPlans.iosevka-www-saffi.ligations]
                enables = [
                  "center-ops",
                  "center-op-trigger-plus-minus-r",
                  "center-op-trigger-equal-l",
                  "center-op-trigger-equal-r",
                  "center-op-trigger-bar-l",
                  "center-op-trigger-bar-r",
                  "center-op-trigger-angle-inside",
                  "center-op-trigger-angle-outside",
                  "center-op-influence-dot",
                  "center-op-influence-colon",
                  "arrow-l",
                  "arrow-r",
                  "counter-arrow-l",
                  "counter-arrow-r",
                  "trig",
                  "eqeqeq",
                  "eqeq",
                  "lteq",
                  "gteq",
                  "exeqeqeq",
                  "exeqeq",
                  "exeq",
                  "eqslasheq",
                  "slasheq",
                  "ltgt-diamond",
                  "ltgt-slash-tag",
                  "slash-asterisk",
                  "plusplus",
                  "kern-dotty",
                  "kern-bars",
                  "logic",
                  "llggeq",
                  "html-comment",
                  "connected-number-sign",
                  "connected-tilde-as-wave",
                ]
                disables = [
                  "center-op-trigger-plus-minus-l",
                  "eqlt",
                  "lteq-separate",
                  "eqlt-separate",
                  "gteq-separate",
                  "eqexeq",
                  "eqexeq-dl",
                  "tildeeq",
                  "ltgt-ne",
                  "ltgt-diamond-tag",
                  "brst",
                  "llgg",
                  "colon-greater-as-colon-arrow",
                  "brace-bar",
                  "brack-bar",
                  "connected-underscore",
                  "connected-hyphen",
                ]

                [buildPlans.iosevka-www-saffi.weights.Regular]
                shape = 400
                menu = 400
                css = 400

                [buildPlans.iosevka-www-saffi.widths.Normal]
                shape = 500
                menu = 5
                css = "normal"

                [buildPlans.iosevka-www-saffi.slopes.Upright]
                angle = 0
                shape = "upright"
                menu = "upright"
                css = "normal"
              '';
            }).overrideAttrs (old: {
              buildPhase = ''
                export HOME=$TMPDIR
                runHook preBuild
                npm run build --no-update-notifier -- --jCmd=$NIX_BUILD_CORES --verbose=9 woff2::$pname
                runHook postBuild
              '';

              installPhase = ''
                runHook preInstall
                fontdir="$out/share/fonts/woff2"
                install -d "$fontdir"
                install "dist/$pname/woff2"/* "$fontdir"
                runHook postInstall
              '';
            });
          })
        ];
      };

      supportedSystems = with flake-utils.lib.system; [
        aarch64-darwin
        x86_64-darwin
        x86_64-linux
      ];

      inherit (flake-utils.lib) eachSystem;
    in
    eachSystem supportedSystems (system:
    let
      pkgs = pkgsFor system;

      rustPkgs = pkgs.rustBuilder.makePackageSet {
        packageFun = import ./Cargo.nix;
        rustToolchain = pkgs.rust-toolchain;
      };

      inherit (pkgs.lib) optionals;
    in
    rec
    {
      packages = rec {
        saffi = (rustPkgs.workspace.www-saffi { }).out;
        saffi-dev = (rustPkgs.workspace.www-saffi-dev { }).bin;
        saffi-wtf = (rustPkgs.workspace.www-saffi-wtf { }).bin;

        saffi-wtf-content = pkgs.stdenv.mkDerivation {
          name = "saffi-wtf-content";
          src = ./saffi-wtf/content;

          phases = "installPhase";
          installPhase = ''
            mkdir -p $out
            cp -vrf $src/* $out
          '';
        };

        saffi-wtf-static = pkgs.stdenv.mkDerivation {
          name = "saffi-wtf-static";
          srcs = [
            ./saffi-wtf/static
            "${pkgs.iosevka}/share/fonts/woff2"
          ];
          sourceRoot = ".";

          phases = [ "unpackPhase" "installPhase" ];

          installPhase = ''
            mkdir -p $out
            cp -vrf static/* $out
            cp -vrf woff2/iosevka-www-saffi-NormalRegularUpright.woff2 $out/iosevka-regular.woff2
          '';
        };
      };

      apps = rec {
        saffi-dev = flake-utils.lib.mkApp {
          drv = packages.saffi-dev;
        };
        saffi-wtf = flake-utils.lib.mkApp {
          drv = packages.saffi-wtf;
        };
      };

      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo2nix
          convco
          nixpkgs-fmt
          rust-toolchain

          libiconv
        ] ++ (optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
          CoreServices
        ]));

        THEMES_PATH = "${onehalf}/sublimetext";
        STATIC_PATH = packages.saffi-wtf-static;
      };

      formatter = pkgs.nixpkgs-fmt;
    });
}
