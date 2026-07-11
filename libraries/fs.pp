// File handles and byte I/O. A `File` holds an OS file descriptor privately;
// `open` produces one from a path, and other libraries adopt descriptors they
// obtained elsewhere (a child's pipe, a socket) through `File.from_fd`, so
// the ordinary read/write/close methods drive any byte stream.
//
// This is a library rather than part of `std`: asking the operating system to
// open and move bytes needs native code, which arrives as a plugin instead of
// a runtime builtin. Point `PREPOLY_INCLUDE` at this directory and import it:
//
//     PREPOLY_INCLUDE=/path/to/libraries
//     import fs.{ File, open, read_file, write_file }
//
// Build the plugin once with `libraries/build.sh`. Sizes are the path
// library's business (a size needs no open file, only a name), so `size`
// works exactly for files opened by path.

import libfs.{
    fd_open,
    fd_read,
    fd_write,
    fd_seek,
    fd_close,
}
import path.{ Path }

/**
 * An open file (or any adopted byte stream): read, write, seek, close. The
 * descriptor and the path it was opened under (empty for adopted descriptors
 * and the standard streams) are private; every capability is a method.
 */
type File = {
    _fd: int64
    _path: string
}

/**
 * Adopt an already-open descriptor as a `File`, so the ordinary
 * read/write/close methods drive it. Where the descriptor came from -- a pipe
 * handed back by a spawned child, a socket -- is the caller's business. The
 * `File` owns the descriptor from here: closing the `File` closes it.
 */
fun File.from_fd(fd: int64) -> File {
    return File { _fd: fd, _path: "" }
}

/**
 * Open the file at `path`. Modes: `"r"` read, `"w"` truncate+create write,
 * `"a"` append+create.
 */
fun open(path: string, mode: string) -> File! {
    return File { _fd: fd_open(path, mode)!, _path: path }
}

/** Standard input as a `File` (never closed by `close`). */
fun File.stdin() -> File {
    return File { _fd: 0, _path: "" }
}

/** Standard output as a `File` (never closed by `close`). */
fun File.stdout() -> File {
    return File { _fd: 1, _path: "" }
}

/** Standard error as a `File` (never closed by `close`). */
fun File.stderr() -> File {
    return File { _fd: 2, _path: "" }
}

/** Block for data and read up to `max` bytes (fewer on a short read; empty at end-of-file). */
fun File.read(self, max: int64) -> uint8[]! {
    return fd_read(self._fd, max)!
}

/** Write all of `data`, returning the byte count written. */
fun File.write(self, data: uint8[]) -> int64! {
    return fd_write(self._fd, data)!
}

/** Move the read/write cursor to absolute byte offset `pos` from the start. Fallible. */
fun File.seek(self, pos: int64) {
    fd_seek(self._fd, pos)!
}

/**
 * The file's length in bytes. Answered by the path library (a stat by name,
 * which needs no open descriptor), so it works exactly for files opened by
 * path -- an adopted descriptor or standard stream has no path to ask about.
 */
fun File.size(self) -> int64! {
    if self._path == "" {
        error("this file was not opened by path; its size is unknown")!
    }
    return Path.parse(self._path).file_size()!
}

/**
 * Close the file. The standard streams are left open, and the stored
 * descriptor is forgotten on close so a second `close()` cannot re-close a
 * descriptor the OS may have reassigned to a later `open` (it is a no-op),
 * and reads/writes after close fail instead of hitting an unrelated file.
 */
fun File.close(self) {
    if self._fd > 2 {
        fd_close(self._fd)!
        self._fd = -1
    }
}

/** Read the whole file at `path` as text. Fallible: returns a `Result`. */
fun read_file(path: string) {
    let f = open(path, "r")!
    let size = f.size()!
    let bytes = f.read(size)!
    f.close()!
    return to_text(bytes)!
}

/** Write `content` to the file at `path`, truncating it. Fallible: returns a `Result`. */
fun write_file(path: string, content: string) {
    let f = open(path, "w")!
    f.write(to_bytes(content))!
    f.close()!
}
