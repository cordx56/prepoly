type Account = {
    owner: string
    balance: int32

    deposit(self, amount: int32) {
        self.balance += amount
    }
}

fun main() {
    let acc = Account { owner: "Alice", balance: 0 }
    acc.deposit(100)
    acc.deposit(50)
    println("{acc.owner}: {acc.balance}")
    println("{acc}")
}
