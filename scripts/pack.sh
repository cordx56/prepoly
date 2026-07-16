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
czm_path="$tmp/bin/czm"
cat << CZM > "$czm_path"
#!/usr/bin/env -S brass --

import package_manager.exec.main

main()
CZM
chmod +x "$czm_path"

#
# libraries
#

./libraries/build.sh release

for path in $(find libraries -type f | grep -e '\.cz$' -e '\.so$'); do
    mkdir -p "$tmp/$(dirname "$path")"
    cp "$path" "$tmp/$path"
done

#
# smoke check
#
# Checking `czm` against the freshly packed bin/ and libraries/ validates the
# archive's contents before they ship. The `.czcache` this writes is NOT
# packed: czm's graph imports native plugins, whose stamps pin the packer's
# temporary absolute paths (the synthesized wrappers dlopen exactly those
# strings), so the cache could never validate on an installed machine; each
# install writes its own on first run instead.
#
env -u BRASS_INCLUDE -u BRASS_PACKAGES "$tmp/bin/brass" check "$tmp/bin/czm"
rm -f "$tmp/bin/czm.czcache"

#
# make tarball
#

cd "$tmp"
find bin libraries -type f | xargs tar czf "$cwd/brass-$(rustc --print=host-tuple).tar.gz"

cd "$cwd"
rm -rf "$tmp"
