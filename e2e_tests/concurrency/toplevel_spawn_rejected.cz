// Module init code never runs through the ownership pass, so a top-level
// spawn would get no promotion or guarding at all; it is rejected.
spawn(() -> {
    println("never")
})
sync()
