// A method on a nominal type is implemented with `fun T.m(...)` rather than
// inside the type body. A first `self` parameter makes it an instance method
// (`acc.deposit(...)`); without `self` it is static (`Account.open(...)`). The
// implementation lives in the same module as the type and needs no import.
type Account = {
    owner: string
    balance: int32
}

fun Account.open(owner: string) {
    return Account { owner: owner, balance: 0 }
}

fun Account.deposit(self, amount: int32) {
    self.balance += amount
    return self.balance
}

fun Account.label(self) {
    return "{self.owner}: {self.balance}"
}

fun main() {
    let acc = Account.open("Alice")
    let total: int32 = acc.deposit(120)
    println("total = {total}")
    println(acc.label())
}
