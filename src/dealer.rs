use num::Zero;
use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};
use num_primes::{Generator, RandBigInt, Verification};

// for demonstration pick 32 bits
const BIT_SIZE: usize = 32;

#[derive(Debug)]
pub struct Dealer {
    p: BigUint,
    q: BigUint,
    g: BigUint,
    a: Vec<BigUint>,
    shares: Vec<(BigUint, BigUint)>,
    c: Vec<BigUint>,
}

impl Dealer {
    fn find_p(q: &BigUint) -> BigUint {
        // find p s.t. p is prime and q | (p - 1)
        let mut k = Generator::new_uint(BIT_SIZE);
        let mut p = k * q + (1 as usize);

        while !Verification::is_prime(&p) {
            k = Generator::new_uint(BIT_SIZE);
            p = k * q + (1 as usize);
        }

        p
    }

    fn find_g(p: &BigUint, q: &BigUint) -> BigUint {
        // find generator of order q in mod p
        // choose any b in [2, p - 2]
        // then g = b ^((p - 1) / q) mod p
        let b =
            rand::thread_rng().gen_biguint_range(&BigUint::from(2 as usize), &(p - (1 as usize)));
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
        a.iter()
            .enumerate()
            .map(|(i, a_i)| a_i * num::pow(BigUint::from(x), i))
            .sum::<BigUint>()
    }

    fn gen_shares(a: &Vec<BigUint>, n: usize, q: &BigUint) -> Vec<(BigUint, BigUint)> {
        // for i = 1..=n, P(i) % q
        (1..=n)
            .into_iter()
            .map(|i| {
                (
                    i.to_biguint().unwrap(),
                    Dealer::eval_poly_at(a, i) % q,
                )
            })
            .collect()
    }

    fn verify(i: usize, s: &BigUint, g: &BigUint, c: &Vec<BigUint>, p: &BigUint) -> bool {
        let share_check = g.modpow(s, p);
        let mut check = BigUint::from(1 as usize);
        for (j, c_i) in c.iter().enumerate() {
            check = (check * c_i.modpow(&(num::pow(BigUint::from(i), j)), p)) % p;
        }

        share_check == check
    }

    fn div_mod_p(a: &BigInt, b: &BigInt, m: &BigUint) -> BigInt {
        let b = if b < &BigInt::zero() { (b % m.to_bigint().unwrap()) + m.to_bigint().unwrap() } else { b.clone() };
        // Finds inverse t , bt congruent-to 1 mod m
        let (mut t_0, mut t_1) = (0.to_bigint().unwrap(), 1.to_bigint().unwrap());
        let (mut r_0, mut r_1) = (m.to_bigint().unwrap(), b);

        while r_1 != 0.to_bigint().unwrap() {
            let q = &r_0 / &r_1;
            (t_0, t_1) = (t_1.clone(), t_0 - &q * t_1);
            (r_0, r_1) = (r_1.clone(), r_0 - &q * r_1);
        }

        if t_0 < BigInt::zero() {
            t_0 = t_0 + m.to_bigint().unwrap();
        }

        a * t_0
    }

    fn reconstruct(shares: &Vec<(BigUint, BigUint)>, q: &BigUint) -> BigUint {
        let mut secret = 0.to_bigint().unwrap();

        for (x_j, y_j) in shares {
            let mut prod = 1.to_bigint().unwrap();

            for (x_m, _) in shares {
                if x_m != x_j {
                    let delta = x_m.to_bigint().unwrap() - x_j.to_bigint().unwrap();
                    prod = (prod * Dealer::div_mod_p(&x_m.to_bigint().unwrap(), &delta, &q))
                        % q.to_bigint().unwrap();
                }
            }

            secret = (secret + (y_j.to_bigint().unwrap() * prod)) % q.to_bigint().unwrap();
        }

        secret.to_biguint().unwrap()
    }

    pub fn new(n: usize, t: usize, secret: usize) -> Dealer {
        // find two primes p, and q s.t. q | p - 1
        let q = Generator::new_prime(BIT_SIZE);
        let p = Dealer::find_p(&q);
        // find generator of order q in multiplicative group p
        let g: BigUint = Dealer::find_g(&p, &q);
        // generate random polynomial of degree t
        let a = [vec![BigUint::from(secret)], vec![Dealer::gen_a(&q); t - 1]].concat();
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
    use num::ToPrimitive;
    use num_bigint::ToBigUint;

    use super::Dealer;

    #[test]
    fn dealer_verify() {
        let n = 5;
        let dealer = Dealer::new(n, 3, 1234);

        for i in 1..=n {
            assert!(Dealer::verify(
                i,
                &dealer.shares[i - 1].1,
                &dealer.g,
                &dealer.c,
                &dealer.p
            ));
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

        assert_eq!(
            1234 as usize,
            Dealer::reconstruct(&shares, &q).to_usize().unwrap()
        );
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
                Dealer::reconstruct(&shares, &dealer.q).to_usize().unwrap(),
                "failed: {:?}\nq: {}",
                shares,
                dealer.q,
            );
        }
    }

    #[test]
    fn dealer_eval_poly_at() {
        let a = vec![
            1.to_biguint().unwrap(),
            2.to_biguint().unwrap(),
            3.to_biguint().unwrap(),
        ];
        // 1 + 2(3) + 3(3)^2 = 1 + 6 + 27 = 34
        let ans = 34.to_biguint().unwrap();

        assert_eq!(ans, Dealer::eval_poly_at(&a, 3));
    }
}
