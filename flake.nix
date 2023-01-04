{
  description = "Emacs Webrender Nix flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    emacs-overlay = {
      type = "github";
      owner = "nix-community";
      repo = "emacs-overlay";
    };
    devshell.url = "github:numtide/devshell";

    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, emacs-overlay, flake-compat, rust-overlay
    , flake-utils, devshell, }:
    { } // (flake-utils.lib.eachSystem [ "x86_64-linux" "x86_64-darwin" ]
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              self.overlays.default
              emacs-overlay.overlay
              (import rust-overlay)
              devshell.overlay
            ];
            config = { };
          };
        in rec {
          devShells.default = with pkgs;
            let custom-llvmPackages = llvmPackages_latest;
            in pkgs.devshell.mkShell {
              imports = [
                ./nix/rust.nix
                (pkgs.devshell.importTOML ./nix/commands.toml)
              ];

              packages = [ custom-llvmPackages.clang ];
              env = [
                {
                  name = "LIBCLANG_PATH";
                  value = "${custom-llvmPackages.libclang}/lib";
                }
                {
                  name = "CACHIX_AUTH_TOKEN";
                  value = let
                    pwd = builtins.getEnv "PWD";
                    key = pwd + "/nix/cachix-key.secrets";
                  in if lib.pathExists key then
                    lib.removeSuffix "\n" (builtins.readFile key)
                  else
                    "";
                }
              ];
            };

          apps = {
            emacsWebrender = flake-utils.lib.mkApp {
              drv = packages.emacsWebrender;
              exePath = "/bin/emacs";
            };
            emacsclient = flake-utils.lib.mkApp {
              drv = packages.emacsWebrender;
              exePath = "/bin/emacsclient";
            };
          };

          defaultApp = apps.emacsWebrender;

          packages = flake-utils.lib.flattenTree {
            inherit (pkgs) emacsWebrender;
            default = pkgs.emacsWebrender;
          };

          hydraJobs = { inherit packages; };
        })) // {
          overlays.default = final: prev:
            let
              #rust nightly date
              emacsWebrenderSources =
                prev.callPackages ./nix/_sources/generated.nix { };
              emacsWebrenderSource = emacsWebrenderSources.emacs-webrender.src;
              locked-date = "2022-10-24";
            in {
              emacsWebrender = with prev;
                let withWebrender = true;
                in (final.emacsGit.override {
                  withImageMagick = true;
                  inherit (prev) imagemagick;
                }).overrideAttrs (old:
                  let
                    custom-llvmPackages = prev.llvmPackages_latest;
                    #withGLX
                    rpathLibs = with xorg;
                      lib.optionals (stdenv.isLinux && withWebrender) [
                        libX11
                        libGLU
                        libGL
                        libXpm
                        libXext
                        libXxf86vm
                        alsaLib
                        libxkbcommon
                        wayland
                        libxcb
                      ];
                  in rec {
                    name = "emacs-webrender-" + version;
                    src = emacsWebrenderSource;
                    version = builtins.substring 0 7 emacsWebrenderSource.rev;
                    # https://github.com/NixOS/nixpkgs/blob/22.11/pkgs/applications/networking/browsers/firefox/common.nix#L574
                    # Firefox use this.
                    # guix has cargo-utils to fix checksum, won't be useful on nix though
                    # https://github.com/ctrlcctrlv/revendor.guile
                    dontFixLibtool = true;

                    preConfigure = (old.preConfigure or "") + "\n"
                      + lib.optionalString withWebrender ''
                        export NIX_CFLAGS_LINK="$NIX_CFLAGS_LINK -lxcb-render -lxcb-xfixes -lxcb-shape"
                      '';

                    patches = (old.patches or [ ]) ++ [ ];

                    makeFlags = (old.makeFlags or [ ]) ++ [
                      "CARGO_FLAGS=--offline" # nightly channel
                    ];

                    #custom configure Flags Setting
                    configureFlags = (if withWebrender then
                      lib.subtractLists [
                        "--with-x-toolkit=gtk3"
                        "--with-xft"
                        "--with-harfbuzz"
                        "--with-cairo"
                        "--with-imagemagick"
                      ] old.configureFlags
                    else
                      old.configureFlags) ++ [
                        "--with-json"
                        "--with-threads"
                        "--with-included-regex"
                        "--with-compress-install"
                        "--with-zlib"
                        "--with-dumping=pdumper"
                      ] ++ lib.optionals withWebrender [ "--with-webrender" ]
                      ++ lib.optionals (stdenv.isDarwin && withWebrender)
                      [ "--disable-webrender-self-contained" ]
                      ++ lib.optionals stdenv.isLinux [ "--with-dbus" ];

                    postPatch = (old.postPatch or "") + ''
                      pwd="$(type -P pwd)"
                      substituteInPlace Makefile.in --replace "/bin/pwd" "$pwd"
                      substituteInPlace lib-src/Makefile.in --replace "/bin/pwd" "$pwd"
                    '';

                    LIBCLANG_PATH = "${custom-llvmPackages.libclang.lib}/lib";
                    RUST_BACKTRACE = "full";

                    buildInputs = (old.buildInputs or [ ]) ++ [
                      custom-llvmPackages.clang
                      custom-llvmPackages.libclang
                      final.rust-bin.nightly."${locked-date}".default
                      git
                    ] ++ lib.optionals withWebrender
                      (with xorg; [ python3 rpathLibs ])
                      ++ lib.optionals stdenv.isDarwin
                      (with darwin.apple_sdk.frameworks;
                        with darwin;
                        [
                          libobjc
                          Security
                          CoreServices
                          Metal
                          Foundation
                          libiconv
                        ] ++ lib.optionals (withWebrender && stdenv.isDarwin) [
                          AppKit
                          CoreGraphics
                          CoreServices
                          CoreText
                          Foundation
                          OpenGL
                        ]);

                    dontPatchShebangs =
                      true; # straight_watch_callback.py: unsupported interpreter directive "#!/usr/bin/env -S python3 -u"

                    # postFixup =
                    #   (old.postFixup or "")
                    #   + (
                    #     if withWebrender
                    #     then
                    #       lib.concatStringsSep "\n" [
                    #         (lib.optionalString stdenv.isLinux ''
                    #           patchelf --set-rpath \
                    #             "$(patchelf --print-rpath "$out/bin/.emacs-29.0.60-wrapped"):${lib.makeLibraryPath rpathLibs}" \
                    #             "$out/bin/.emacs-29.0.60-wrapped"
                    #             patchelf --add-needed "libfontconfig.so" "$out/bin/.emacs-29.0.60-wrapped"
                    #         '')
                    #       ]
                    #     else ""
                    #   );
                  });
            };
        };
}
