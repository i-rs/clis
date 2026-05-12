#!/bin/bash
set -e

DIST_DIR="target/distrib"
TAP_REPO="i-rs/homebrew-tap"
PACKAGES=("i-rs-hello" "i-rs-fs" "i-rs-http")

echo "Cloning homebrew-tap repository..."
rm -rf /tmp/homebrew-tap
git clone "https://github.com/$TAP_REPO.git" /tmp/homebrew-tap

for pkg in "${PACKAGES[@]}"; do
  echo "Publishing $pkg to homebrew..."

  FORMULA_FILE="$DIST_DIR/$pkg.rb"

  if [ ! -f "$FORMULA_FILE" ]; then
    echo "Warning: $FORMULA_FILE not found, skipping..."
    continue
  fi

  cp "$FORMULA_FILE" "/tmp/homebrew-tap/$pkg.rb"

  cd /tmp/homebrew-tap
  git add "$pkg.rb"
  git commit -m "Update $pkg to latest version"
  git push "https://$(git config user.email)@github.com/$TAP_REPO.git" HEAD:main
  cd - > /dev/null

  echo "$pkg published to homebrew-tap!"
done

echo "All homebrew formulas updated!"
echo "Please merge the PRs at https://github.com/$TAP_REPO/pulls"
