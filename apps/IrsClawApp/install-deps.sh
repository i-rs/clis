#!/usr/bin/env bash
set -euo pipefail

PROJECT="IrsClawApp.xcodeproj"
SCHEME="IrsClawApp"

step() { printf "\033[1;34m→\033[0m %s\n" "$1"; }
ok()   { printf "\033[1;32m✓\033[0m %s\n" "$1"; }
die()  { printf "\033[1;31m✗\033[0m %s\n" "$1" >&2; exit 1; }

cd "$(dirname "$0")"

if ! command -v xcodebuild &>/dev/null; then
    die "xcodebuild not found. Install Xcode CLI tools first: xcode-select --install"
fi

step "Resetting SPM cache..."
rm -rf ~/Library/Caches/org.swift.swiftpm/repositories/{NetworkImage,swift-markdown-ui,swift-cmark}-*
rm -rf ~/Library/Developer/Xcode/DerivedData/IrsClawApp-*
rm -rf IrsClawApp.xcodeproj/project.xcworkspace/xcshareddata/swiftpm/Package.resolved
rm -rf .build
ok "Cache cleared"

step "Resolving SPM dependencies..."
xcodebuild \
    -project "$PROJECT" \
    -scheme "$SCHEME" \
    -resolvePackageDependencies \
    -clonedSourcePackagesDirPath .spm-clones \
    2>&1 | tail -5

if [ $? -eq 0 ]; then
    ok "Dependencies resolved"
else
    die "Failed to resolve dependencies"
fi

step "Building (validation)..."
xcodebuild \
    -project "$PROJECT" \
    -scheme "$SCHEME" \
    -destination 'platform=macOS' \
    -quiet \
    build \
    2>&1 | tail -5

if [ $? -eq 0 ]; then
    ok "Build succeeded"
else
    die "Build failed"
fi

printf "\n\033[1;32mDone. Open the project in Xcode:\033[0m  open %s\n" "$PROJECT"
