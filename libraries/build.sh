#!/bin/bash -e
#
# Usage: libraries/build.sh [profile]

cd "$(dirname "$0")/.."

profile="${1:-"release"}"

case "$(uname -s)" in
Darwin) suffix=dylib ;;
MINGW* | MSYS* | CYGWIN*) suffix=dll ;;
*) suffix=so ;;
esac

for dirpath in $(find libraries -type f | grep "Cargo.toml$" | xargs -L 1 dirname); do
    libname="$(basename "$dirpath")"
    pkgname="brass_lib_$libname"
    cargo build -p "$pkgname" --profile="$profile"

    built="target/$profile/lib$libname.$suffix"
    [[ "$suffix" == dll ]] && built="target/$profile/$libname.$suffix"

    basedir="$(dirname "$dirpath")"
    mkdir -p "$basedir"
    cp "$built" "$basedir/lib$libname.$suffix"
    echo "lib$libname built"
done
