let
  pkgs = import ../.;
  inherit (pkgs) lib;

in rec {

  # We generate Nix code to get an attribute set with the component names as
  # keys and lists of the corresponding components as values

  testsListsIfd = pkgs.runCommand "unittests-list" {} ''
    cd ${pkgs.qaul-rust.testBinaries}/bin

    echo "{" > $out
    for binary in *
    do
      echo "  \"''${binary%-test}\" = [" >> $out
      ./$binary --list | sed -n "s/\(.*\): test/    \"\1\"/p" >> $out
      echo "  ];" >> $out
    done
    echo "}" >> $out
  '';

  testsLists = import testsListsIfd;


  # provide a nested attribute set containing a derivation for each test:
  #
  # $ nix-build nix/ci/rust.nix -A tests.trivial.batch_insert
  # $ ls result
  # log status
  #
  # $

  tests = with lib; mapAttrs (
    component: tests: listToAttrs (
      map (
        test: nameValuePair test (
          pkgs.runCommand "${component}-${test}" {} ''
            set -xo pipefail
            mkdir -p $out
            ${pkgs.qaul-rust.testBinaries}/bin/${component}-test ${test} | tee $out/log
            echo $? > $out/status
            set +x
          ''
        )
      ) tests
    )
  ) testsLists;


  # Generate a GitLab CI pipeline file

  jobs = with lib; listToAttrs (
    concatLists (
      mapAttrsToList (
        component: tests: (
          map (
            test: nameValuePair "${component}-${test}" {
              needs = [];
              tags = [ "irdest-nix" ];
              stage = "build";
              script = [
                "nix-build nix/ci/rust.nix -A tests.${component}.${test}"
                "exit $(cat result/status)"
              ];
              artifacts = {
                when = "always";
                paths = [ "result/*" ];
              };
            }
          ) tests
        )
      ) testsLists
    )
  );

  pipeline = pkgs.writeText "rust-gitlab-ci.yml" (builtins.toJSON jobs);
}
