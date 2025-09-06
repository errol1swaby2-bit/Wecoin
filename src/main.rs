use wecoin::{Transaction, Block};

fn main() {
    let tx1 = Transaction {
        from: "Alice".to_string(),
        to: "Bob".to_string(),
        amount: 50,
    };

    let tx2 = Transaction {
        from: "Bob".to_string(),
        to: "Charlie".to_string(),
        amount: 25,
    };

    let block = Block::new(1, vec![tx1, tx2], "genesis".to_string());

    println!("Block created:\n{:#?}", block);
}
