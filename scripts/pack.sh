#!/bin/bash -e

cd "$(dirname "$0")/../"
cwd="$(pwd)"
tmp="$(mktemp -d)"

./x cargo install --path crates/brass_driver --root "$tmp"
./x cargo install --path crates/brass_language_server --root "$tmp"
./x cargo install --path crates/brass_formatter --root "$tmp"

#
# Brass scripts
#
czpm_path="$tmp/bin/czpm"
cat << CZPM > "$czpm_path"
#!/usr/bin/env -S brass --

import package_manager.exec.main

main()
CZPM
chmod +x "$czpm_path"

#
# libraries
#

./libraries/build.sh release

for path in $(find libraries -type f | grep -e '\.cz$' -e '\.so$'); do
    mkdir -p "$tmp/$(dirname "$path")"
    cp "$path" "$tmp/$path"
done

env -u BRASS_INCLUDE -u BRASS_PACKAGES "$tmp/bin/brass" check "$tmp/bin/czpm"
#rm -f "$tmp/bin/czpm.czcache"

#
# make tarball
#

cd "$tmp"
find bin libraries -type f | xargs tar czf "$cwd/brass-$(rustc --print=host-tuple).tar.gz"

cd "$cwd"
rm -rf "$tmp"
