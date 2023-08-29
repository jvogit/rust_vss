use num_bigint::BigUint;
use num_primes::{Generator, RandBigInt, Verification};

use crate::vss;

// for demonstration pick 32 bits
const BIT_SIZE: usize = 32;

#[derive(Debug)]
pub struct Dealer {
    pub p: BigUint,
    pub q: BigUint,
    pub g: BigUint,
    pub shares: Vec<(BigUint, BigUint)>,
    pub c: Vec<BigUint>,
    pub t: usize,
    pub n: usize,
}

impl Dealer {
    /// Given a prime q, find a prime p s.t. q | (p - 1)
    fn find_p(q: &BigUint) -> BigUint {
        let mut k = Generator::new_uint(BIT_SIZE);
        let mut p = k * q + (1 as usize);

        while !Verification::is_prime(&p) {
            k = Generator::new_uint(BIT_SIZE);
            p = k * q + (1 as usize);
        }

        p
    }

    /// Find generator of order q in prime field p
    ///
    /// choose any b in [2, p - 2] then g = b ^((p - 1) / q) mod p
    fn find_g(p: &BigUint, q: &BigUint) -> BigUint {
        let b =
            rand::thread_rng().gen_biguint_range(&BigUint::from(2 as usize), &(p - (1 as usize)));
        let e = (p - (1 as u32)) / q;

        b.modpow(&e, p)
    }

    /// Generate polynomial coefficients in primefield q
    fn gen_a(q: &BigUint) -> BigUint {
        rand::thread_rng().gen_biguint_below(q)
    }

    /// Return a new Dealer
    pub fn new(n: usize, t: usize, secret: usize) -> Dealer {
        // find two primes p, and q s.t. q | p - 1
        let q = Generator::new_prime(BIT_SIZE);
        let p = Dealer::find_p(&q);
        // find generator of order q in multiplicative group p
        let g: BigUint = Dealer::find_g(&p, &q);
        // generate random polynomial of degree t
        let a = [vec![BigUint::from(secret)], vec![Dealer::gen_a(&q); t - 1]].concat();
        // generate commitments
        let c = vss::generate_commitments(&a, &g, &p);
        // generate shares
        let shares = vss::generate_shares(&a, n, &q);

        Dealer {
            p,
            q,
            g,
            shares,
            c,
            t,
            n,
        }
    }
}

#[cfg(test)]
mod tests {
    use num::ToPrimitive;
    use num_bigint::ToBigUint;

    use crate::vss;

    use super::Dealer;

    #[test]
    fn dealer_verify() {
        let n = 5;
        let dealer = Dealer::new(n, 3, 1234);

        for (i, s_i) in dealer.shares {
            assert!(vss::verify_share(&i, &s_i, &dealer.g, &dealer.c, &dealer.p));
        }
    }

    #[test]
    fn dealer_reconstruct() {
        let shares = vec![
            (2.to_biguint().unwrap(), 1942.to_biguint().unwrap()),
            (4.to_biguint().unwrap(), 3402.to_biguint().unwrap()),
            (5.to_biguint().unwrap(), 4414.to_biguint().unwrap()),
        ];
        // random prime
        let q = 13931.to_biguint().unwrap();

        assert_eq!(1234, vss::reconstruct(&shares, &q).to_usize().unwrap());
    }

    #[test]
    fn dealer_reconstruct_shares() {
        let dealer = Dealer::new(5, 3, 1234);
        let k_shares = vec![
            vec![
                dealer.shares[0].clone(),
                dealer.shares[1].clone(),
                dealer.shares[2].clone(),
            ],
            vec![
                dealer.shares[0].clone(),
                dealer.shares[2].clone(),
                dealer.shares[3].clone(),
            ],
            vec![
                dealer.shares[1].clone(),
                dealer.shares[2].clone(),
                dealer.shares[3].clone(),
            ],
            vec![
                dealer.shares[4].clone(),
                dealer.shares[2].clone(),
                dealer.shares[0].clone(),
            ],
        ];

        for shares in k_shares {
            assert_eq!(
                1234,
                vss::reconstruct(&shares, &dealer.q).to_usize().unwrap(),
                "failed: {:?}\nq: {}",
                shares,
                dealer.q,
            );
        }
    }
}
