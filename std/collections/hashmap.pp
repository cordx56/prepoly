// A hash map with open-addressing (linear-probing) storage, written in Prepoly on
// the runtime string/array primitives. Keys may be of any type that renders to a
// stable string and compares with `==` (integers, strings, records, ...); values
// may be of any type. Part of the standard library.
//
// Construction takes two *witness* values rather than being empty by default:
// Prepoly has no generic type parameters, and the typed back end fixes an array's
// element type from a concretely-typed element in the constructing function. An
// empty `new()` could not pin the key/value types, so `new(sample_key,
// sample_value)` uses the samples only to fix those types -- they are never stored,
// and the returned map is empty. The samples must have the same types as the keys
// and values later inserted (e.g. `HashMap.new("", 0)` for a `string -> int32`
// map; note `0` is `int32`, so use an `int64` witness for `int64` values).

// One stored key/value pair. Private to this module: it is an implementation
// detail of the table's slot array, not part of the public surface.
type _Entry = {
    key
    value
}

type HashMap = {
    // Slot arrays, parallel and `cap`-long. `entries[i]` is meaningful only when
    // `states[i]` is `_FULL`. A slot is `_EMPTY` (never used), `_FULL` (holds a
    // live pair), or `_TOMB` (deleted -- probing passes through it, insertion may
    // reuse it). `entries` is seeded at construction with witness pairs so its
    // element type is pinned even while every slot is logically empty.
    entries
    states: int32[]
    // `cap` is the slot count (a power of two is not required; the table grows by
    // doubling). `count` is the number of live pairs; `tombs` the number of
    // tombstones. The table grows when `count + tombs` reaches 3/4 of `cap`, which
    // keeps an empty slot present so every probe terminates.
    cap: int64
    count: int64
    tombs: int64

    // An empty map whose key/value types are fixed by the witness samples (which
    // are not stored). See the module comment for why the witnesses are required.
    new(sample_key, sample_value) {
        let cap: int64 = 8
        let zero: int64 = 0
        let entries = []
        let states = []
        let i: int64 = 0
        while i < cap {
            entries.push(_Entry { key: sample_key, value: sample_value })
            states.push(0)
            i += 1
        }
        return Self { entries: entries, states: states, cap: cap, count: zero, tombs: zero }
    }

    // A map built from an array of `[key, value]` pairs. The array must be
    // non-empty: its first pair supplies the witness types `new` needs.
    from_pairs(pairs) {
        let m = Self.new(pairs[0][0], pairs[0][1])
        for p in pairs {
            m.set(p[0], p[1])
        }
        return m
    }

    // The home slot for `key`: an FNV-style polynomial hash over the bytes of the
    // key's string rendering, reduced into `[0, cap)`. `string.from` renders any
    // value, so the table is key-type agnostic; the per-byte modulus keeps the
    // accumulator in a non-negative 31-bit range, so the final index is in range.
    _hash(self, key) -> int64 {
        let bytes = _string_bytes(string.from(key))
        let h: int64 = 2166136261
        for b in bytes {
            h = (h * 16777619 + b) % 2147483647
        }
        return h % self.cap
    }

    // The index of `key` if present, else -1. Linear probing stops at the first
    // `_EMPTY` slot: under this scheme a present key always sits before the first
    // empty slot in its probe sequence, so an empty slot proves absence.
    _find(self, key) -> int64 {
        let one: int64 = 1
        let absent = 0 - one
        let h = self._hash(key)
        let step: int64 = 0
        while step < self.cap {
            let idx = (h + step) % self.cap
            let s = self.states[idx]
            if s == 0 {
                return absent
            }
            if s == 1 {
                if self.entries[idx].key == key {
                    return idx
                }
            }
            step += 1
        }
        return absent
    }

    // Place `key`/`value` into the first non-`_FULL` slot of `key`'s probe
    // sequence. The caller guarantees `key` is absent and a free slot exists (the
    // load factor keeps one), so this only inserts -- it never updates. Reusing a
    // tombstone reclaims it.
    _insert(self, key, value) {
        let one: int64 = 1
        let h = self._hash(key)
        let step: int64 = 0
        while step < self.cap {
            let idx = (h + step) % self.cap
            if self.states[idx] != 1 {
                if self.states[idx] == 2 {
                    self.tombs -= one
                }
                self.entries[idx] = _Entry { key: key, value: value }
                self.states[idx] = 1
                self.count += one
                return
            }
            step += 1
        }
    }

    // Rehash every live pair into a fresh `new_cap`-slot table, dropping tombstones.
    // A live old entry supplies the witness types for the new slot array.
    _grow(self, new_cap) {
        let zero: int64 = 0
        let old_entries = self.entries
        let old_states = self.states
        let old_cap = self.cap
        let witness = old_entries[0]
        let entries = []
        let states = []
        let i: int64 = 0
        while i < new_cap {
            entries.push(_Entry { key: witness.key, value: witness.value })
            states.push(0)
            i += 1
        }
        self.entries = entries
        self.states = states
        self.cap = new_cap
        self.count = zero
        self.tombs = zero
        let j: int64 = 0
        while j < old_cap {
            if old_states[j] == 1 {
                self._insert(old_entries[j].key, old_entries[j].value)
            }
            j += 1
        }
    }

    // Insert `key`/`value`, or overwrite the value if `key` is already present.
    set(self, key, value) {
        let existing = self._find(key)
        if existing >= 0 {
            self.entries[existing].value = value
            return
        }
        // Grow before inserting a new key once the table is 3/4 full, so a free
        // slot always remains and probing terminates.
        if (self.count + self.tombs) * 4 >= self.cap * 3 {
            self._grow(self.cap * 2)
        }
        self._insert(key, value)
    }

    // The value for `key`, or `null` if absent.
    get(self, key) {
        let idx = self._find(key)
        if idx >= 0 {
            return self.entries[idx].value
        }
        return null
    }

    // The value for `key`, or `dflt` if absent. Unlike `get` this never returns a
    // nullable, so the result is usable without a null check.
    get_or(self, key, dflt) {
        let idx = self._find(key)
        if idx >= 0 {
            return self.entries[idx].value
        }
        return dflt
    }

    contains_key(self, key) -> bool {
        return self._find(key) >= 0
    }

    // Remove `key`, returning whether it was present. The slot becomes a tombstone
    // so probes for other keys still reach past it; tombstones are cleared on grow.
    delete(self, key) -> bool {
        let one: int64 = 1
        let idx = self._find(key)
        if idx < 0 {
            return false
        }
        self.states[idx] = 2
        self.tombs += one
        self.count -= one
        return true
    }

    size(self) -> int64 {
        return self.count
    }

    is_empty(self) -> bool {
        return self.count == 0
    }

    // The live keys, in unspecified (slot) order.
    keys(self) {
        let result = []
        let i: int64 = 0
        while i < self.cap {
            if self.states[i] == 1 {
                result.push(self.entries[i].key)
            }
            i += 1
        }
        return result
    }

    // The live values, in the same order as `keys`.
    values(self) {
        let result = []
        let i: int64 = 0
        while i < self.cap {
            if self.states[i] == 1 {
                result.push(self.entries[i].value)
            }
            i += 1
        }
        return result
    }

    // The live pairs as `[key, value]` tuples, in the same order as `keys`.
    pairs(self) {
        let result = []
        let i: int64 = 0
        while i < self.cap {
            if self.states[i] == 1 {
                result.push([self.entries[i].key, self.entries[i].value])
            }
            i += 1
        }
        return result
    }

    // Remove every pair, keeping the current capacity and key/value types.
    clear(self) {
        let zero: int64 = 0
        let i: int64 = 0
        while i < self.cap {
            self.states[i] = 0
            i += 1
        }
        self.count = zero
        self.tombs = zero
    }
}
