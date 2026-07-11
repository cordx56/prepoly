// seek repositions the cursor for the next read, and size answers through
// the path library (a stat by name), so both are exercised through one file.
import fs.{ File, open, write_file }

fun main() {
    const path = "/tmp/prepoly_e2e_seek_and_size.txt"
    write_file(path, "0123456789")!
    let f = open(path, "r")!
    println(f.size()!)
    let first = f.read(4)!
    println(to_text(first)!)
    f.seek(2)!
    let again = f.read(3)!
    println(to_text(again)!)
    f.close()!
}
