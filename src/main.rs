use rust_vss::dealer::Dealer;

fn main() {
    let dealer = Dealer::new(5, 4, 1234);
    
    println!("{:?}", dealer)
}
