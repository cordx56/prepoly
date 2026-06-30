// Structural subtyping: `run_with` places no nominal constraint on `logger`;
// it just calls `.log(msg)`. Any record with a matching method fits, and the
// concrete type is resolved when the value actually arrives (deferred
// monomorphization).

type ConsoleLogger = {
    prefix: string
}

fun ConsoleLogger.log(self, msg: string) {
    println("[{self.prefix}] {msg}")
}

type TaggedLogger = {
    prefix: string
    tag: string
}

fun TaggedLogger.log(self, msg: string) {
    println("[{self.prefix}/{self.tag}] {msg}")
}

fun run_with(logger, task: string) {
    logger.log("starting {task}")
    logger.log("done {task}")
}

fun main() {
    let console = ConsoleLogger { prefix: "APP" }
    let tagged = TaggedLogger { prefix: "APP", tag: "net" }
    run_with(console, "task1")
    run_with(tagged, "task2")
}
