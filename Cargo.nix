# This file was @generated by cargo2nix 0.11.0.
# It is not intended to be manually edited.

args@{ release ? true
, rootFeatures ? [
    "www-saffi/default"
    "www-saffi-dev/default"
    "www-saffi-wtf/default"
  ]
, rustPackages
, buildRustPackages
, hostPlatform
, hostPlatformCpu ? null
, hostPlatformFeatures ? [ ]
, target ? null
, codegenOpts ? null
, profileOpts ? null
, cargoUnstableFlags ? null
, rustcLinkFlags ? null
, rustcBuildFlags ? null
, mkRustCrate
, rustLib
, lib
, workspaceSrc
, ignoreLockHash
,
}:
let
  nixifiedLockHash = "8bb60b4d11c02a16c0fc0ede9fc3123ec8c42e823fbd9cf22751065309b534e9";
  workspaceSrc = if args.workspaceSrc == null then ./. else args.workspaceSrc;
  currentLockHash = builtins.hashFile "sha256" (workspaceSrc + /Cargo.lock);
  lockHashIgnored =
    if ignoreLockHash
    then builtins.trace "Ignoring lock hash" ignoreLockHash
    else ignoreLockHash;
in
if !lockHashIgnored && (nixifiedLockHash != currentLockHash) then
  throw ("Cargo.nix ${nixifiedLockHash} is out of sync with Cargo.lock ${currentLockHash}")
else
  let
    inherit (rustLib) fetchCratesIo fetchCrateLocal fetchCrateGit fetchCrateAlternativeRegistry expandFeatures decideProfile genDrvsByProfile;
    profilesByName = { };
    rootFeatures' = expandFeatures rootFeatures;
    overridableMkRustCrate = f:
      let
        drvs = genDrvsByProfile profilesByName ({ profile, profileName }: mkRustCrate ({ inherit release profile hostPlatformCpu hostPlatformFeatures target profileOpts codegenOpts cargoUnstableFlags rustcLinkFlags rustcBuildFlags; } // (f profileName)));
      in
      { compileMode ? null, profileName ? decideProfile compileMode release }:
      let drv = drvs.${profileName}; in if compileMode == null then drv else drv.override { inherit compileMode; };
  in
  {
    cargo2nixVersion = "0.11.0";
    workspace = {
      www-saffi = rustPackages.unknown.www-saffi."0.1.0";
      www-saffi-dev = rustPackages.unknown.www-saffi-dev."0.1.0";
      www-saffi-wtf = rustPackages.unknown.www-saffi-wtf."0.1.0";
    };
    "unknown".www-saffi."0.1.0" = overridableMkRustCrate (profileName: rec {
      name = "www-saffi";
      version = "0.1.0";
      registry = "unknown";
      src = fetchCrateLocal (workspaceSrc + "/www-saffi");
    });

    "unknown".www-saffi-dev."0.1.0" = overridableMkRustCrate (profileName: rec {
      name = "www-saffi-dev";
      version = "0.1.0";
      registry = "unknown";
      src = fetchCrateLocal (workspaceSrc + "/www-saffi-dev");
      dependencies = {
        www_saffi = (rustPackages."unknown".www-saffi."0.1.0" { inherit profileName; }).out;
      };
    });

    "unknown".www-saffi-wtf."0.1.0" = overridableMkRustCrate (profileName: rec {
      name = "www-saffi-wtf";
      version = "0.1.0";
      registry = "unknown";
      src = fetchCrateLocal (workspaceSrc + "/www-saffi-wtf");
      dependencies = {
        www_saffi = (rustPackages."unknown".www-saffi."0.1.0" { inherit profileName; }).out;
      };
    });

  }
