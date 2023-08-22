use num::ToPrimitive;
use num_primes::{Generator, BigUint, Verification, RandBigInt};

// for demonstration pick 32 bits
const BIT_SIZE: usize = 32;

#[derive(Debug)]
pub struct Dealer {
    p: BigUint,
    q: BigUint,
    g: BigUint,
    a: Vec<BigUint>,
    shares: Vec<BigUint>,
    c: Vec<BigUint>,
}

impl Dealer {
    fn find_p(q: &BigUint) -> BigUint {
        // find p s.t. p is prime and q | (p - 1)
        let mut k = Generator::new_uint(BIT_SIZE);
        let mut p = k*q + (1 as usize);

        while !Verification::is_prime(&p) {
            k = Generator::new_uint(BIT_SIZE);
            p = k*q + (1 as usize);
        }

        p
    }

    fn find_g(p: &BigUint, q: &BigUint) -> BigUint {
        // find generator of order q in mod p
        // choose any b in [2, p - 2]
        // then g = b ^((p - 1) / q) mod p
        let b = rand::thread_rng().gen_biguint_range(&BigUint::from(2 as usize), &(p - (1 as usize)));
        let e = (p - (1 as u32)) / q;

        b.modpow(&e, p)
    }

    fn gen_a(q: &BigUint) -> BigUint {
        rand::thread_rng().gen_biguint_below(q)
    }

    fn gen_c(a: &Vec<BigUint>, g: &BigUint, p: &BigUint) -> Vec<BigUint> {
        a.iter().map(|a_i| g.modpow(a_i, p)).collect()
    }

    fn eval_poly_at(a: &Vec<BigUint>, x: usize) -> BigUint {
        a.iter().enumerate().map(|(i, a_i)| a_i * num::pow(BigUint::from(x), i)).sum::<BigUint>()
    }

    fn gen_shares(a: &Vec<BigUint>, n: usize, q: &BigUint) -> Vec<BigUint> {
        // for i = 1..=n, P(i) % q 
        (1..=n).into_iter().map(|i| Dealer::eval_poly_at(a, i) % q).collect()
    }

    fn verify(i: usize, s: &BigUint, g: &BigUint, c: &Vec<BigUint>, p: &BigUint) -> bool {
        let share_check = g.modpow(s, p);
        let mut check = BigUint::from(1 as usize);
        for (j, c_i) in c.iter().enumerate() {
            check = (check * c_i.modpow(&(num::pow(BigUint::from(i), j)), p)) % p;  
        }

        share_check == check
    }

    pub fn new(n: usize, t: usize, secret: usize) -> Dealer {
        // find two primes p, and q s.t. q | p - 1
        let q = Generator::new_prime(BIT_SIZE);
        let p = Dealer::find_p(&q);
        // find generator of order q in multiplicative group p
        let g: BigUint = Dealer::find_g(&p, &q);
        // generate random polynomial of degree t
        let a = [vec![BigUint::from(secret)], vec![Dealer::gen_a(&q); t]].concat();
        // generate commitments
        let c = Dealer::gen_c(&a, &g, &p);
        // generate shares
        let shares = Dealer::gen_shares(&a, n, &q);

        Dealer {
            p,
            q,
            g,
            a,
            shares,
            c,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Dealer;

    #[test]
    fn dealer_verify() {
        let n = 5;
        let dealer = Dealer::new(n, 3, 1234);

        for i in 1..=n {
            assert!(Dealer::verify(i, &dealer.shares[i - 1], &dealer.g, &dealer.c, &dealer.p));
        }
    }
}
