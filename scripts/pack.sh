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
# analysis caches
#
# `czm` ships with its `.czcache`: checking it against the freshly packed bin/
# and libraries/ writes bin/czm.czcache whose source stamps are RELATIVE to the
# implicit <bin>/../libraries include root, so the cache validates wherever the
# archive is unpacked. The packer's own resolution environment is cleared so no
# machine-local root leaks into the stamps. On a release build the compiler tag
# is portable (channel + commit); a local pack still writes a cache, but only
# the packing machine's own binary would accept it.
#
env -u BRASS_INCLUDE -u BRASS_PACKAGES "$tmp/bin/brass" check "$tmp/bin/czm"

#
# make tarball
#

cd "$tmp"
find bin libraries -type f | xargs tar czf "$cwd/brass-$(rustc --print=host-tuple).tar.gz"

cd "$cwd"
rm -rf "$tmp"
