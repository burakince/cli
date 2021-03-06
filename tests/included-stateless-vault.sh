#!/bin/bash

snapshot="$fixture/snapshots/vault/stateless"

title "'vault' subcommand"
(with "a minimal vault configuration file"
  it "succeeds even if there is no further argument" && \
      echo 'secrets: .' | expect_run $SUCCESSFULLY "$exe" vault -c -
)

title "'vault init' subcommand"

(with "an invalid vault path"
  it "fails" && \
      WITH_SNAPSHOT="$snapshot/invalid-vault-path" \
      expect_run $WITH_FAILURE "$exe" vault -c / init
)

title "'completions' subcommand"

(with "a supported $SHELL"
    it "generates a script executable by $SHELL" && \
      expect_run $SUCCESSFULLY "$exe" completions | $SHELL
)

(with "an explicit supported shell name"
    it "generates a valid script" && \
      expect_run $SUCCESSFULLY "$exe" completions bash | bash
)

(with "an unsupported shell"
    it "fails with a suitable error" && {
      WITH_SNAPSHOT="$snapshot/unsupported-shell" \
      expect_run $WITH_FAILURE "$exe" completions foobar
    }
)
