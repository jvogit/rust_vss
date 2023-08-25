use num::Zero;
use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};

/// Given a polynomial constants a_0,a_1,...a_k, construct a polynomial P over prime field q
/// and evaluate n unique shares
///
/// Shares are in the form (1, P(1)),(2, P(2)),...(n, P(n))
pub fn generate_shares(a: &Vec<BigUint>, n: usize, q: &BigUint) -> Vec<(BigUint, BigUint)> {
    // for i = 1..=n, P(i) % q
    (1..=n)
        .into_iter()
        .map(|i| (i.to_biguint().unwrap(), eval_poly_at(a, i) % q))
        .collect()
}

/// Verify a particular share: (i, s) given generator g, commitments c, and p
///
/// Verifies that g^s is congruent to product of c_0,c_1^(i^1),c_2^(i^2),...,c_n^(i^n) mod p
pub fn verify_share(i: &BigUint, s: &BigUint, g: &BigUint, c: &Vec<BigUint>, p: &BigUint) -> bool {
    let share_check = g.modpow(s, p);
    let mut check = 1.to_biguint().unwrap();

    for (j, c_i) in c.iter().enumerate() {
        let exp = num::pow(i.clone(), j);
        check = (check * c_i.modpow(&exp, p)) % p;
    }

    share_check == check
}

/// Reconstructs the polynomial, P, given shares and q and returns the secret which is P(0)
///
/// Uses Lagrange Interpolating Polynomial Thereom to reconstruct a unique polynomial of degree k given k + 1 unique shares
/// over prime field q
/// https://en.wikipedia.org/wiki/Lagrange_polynomial
/// https://en.wikipedia.org/wiki/Shamir%27s_secret_sharing
pub fn reconstruct(shares: &Vec<(BigUint, BigUint)>, q: &BigUint) -> BigUint {
    let mut secret = 0.to_bigint().unwrap();

    for (x_j, y_j) in shares {
        let mut prod = 1.to_bigint().unwrap();

        for (x_m, _) in shares {
            if x_m != x_j {
                let delta = x_m.to_bigint().unwrap() - x_j.to_bigint().unwrap();
                prod = (prod * div_mod_p(&x_m.to_bigint().unwrap(), &delta, &q))
                    % q.to_bigint().unwrap();
            }
        }

        secret = (secret + (y_j.to_bigint().unwrap() * prod)) % q.to_bigint().unwrap();
    }

    secret.to_biguint().unwrap()
}

/// Generate commitments c given polynomial and generator g of order q mod p
///
/// Commitments are of the form g^a_0 mod p,g^a_1 mod p,...,g^a_n mod p
pub fn generate_commitments(a: &Vec<BigUint>, g: &BigUint, p: &BigUint) -> Vec<BigUint> {
    a.iter().map(|a_i| g.modpow(a_i, p)).collect()
}

/// Evaluates a polynomial, P, from polynomial constants, a, and evaluates P(x)
fn eval_poly_at(a: &Vec<BigUint>, x: usize) -> BigUint {
    a.iter()
        .enumerate()
        .map(|(i, a_i)| a_i * num::pow(BigUint::from(x), i))
        .sum::<BigUint>()
}

/// Evaluates a/b mod p
///
/// Finds inverse of b mod p, t, then returns a*t
fn div_mod_p(a: &BigInt, b: &BigInt, m: &BigUint) -> BigInt {
    // ensure 0 < b < m
    let b = if b < &BigInt::zero() {
        (b % m.to_bigint().unwrap()) + m.to_bigint().unwrap()
    } else {
        b.clone()
    };
    // Finds inverse t , bt congruent-to 1 mod m using extended Eucldian algorithm
    // https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm
    let (mut t_0, mut t_1) = (0.to_bigint().unwrap(), 1.to_bigint().unwrap());
    let (mut r_0, mut r_1) = (m.to_bigint().unwrap(), b);

    while r_1 != 0.to_bigint().unwrap() {
        let q = &r_0 / &r_1;
        (t_0, t_1) = (t_1.clone(), t_0 - &q * t_1);
        (r_0, r_1) = (r_1.clone(), r_0 - &q * r_1);
    }

    // ensure inverse is always positive
    if t_0 < BigInt::zero() {
        t_0 = t_0 + m.to_bigint().unwrap();
    }

    a * t_0
}

#[cfg(test)]
mod tests {
    use num::ToPrimitive;
    use num_bigint::ToBigUint;

    use crate::vss;

    #[test]
    fn generate_shares() {
        let a = vec![
            1.to_biguint().unwrap(),
            2.to_biguint().unwrap(),
            3.to_biguint().unwrap(),
        ];
        let n = 4;
        let q = 5.to_biguint().unwrap();
        let expected_shares = vec![
            // 1 + 2(1) + 3(1^2) = 6 mod 5 = 1 mod 5
            (1.to_biguint().unwrap(), 1.to_biguint().unwrap()),
            // 1 + 2(2) + 3(2^2) = 17 mod 5 = 2 mod 5
            (2.to_biguint().unwrap(), 2.to_biguint().unwrap()),
            // 1 + 2(3) + 3(3^2) = 34 mod 5 = 4 mod 5
            (3.to_biguint().unwrap(), 4.to_biguint().unwrap()),
            // 1 + 2(4) + 3(4^2) = 57 mod 5 = 2 mod 5
            (4.to_biguint().unwrap(), 2.to_biguint().unwrap()),
        ];
        let actual_shares = vss::generate_shares(&a, n, &q);

        assert_eq!(expected_shares, actual_shares);
    }

    #[test]
    fn verify() {
        let a = vec![
            0.to_biguint().unwrap(),
            3.to_biguint().unwrap(),
            4.to_biguint().unwrap(),
        ];
        let p = 11.to_biguint().unwrap();
        let q = 5.to_biguint().unwrap();
        let g = 3.to_biguint().unwrap();
        let c = vec![
            // g^a_0 = 3^0 = 1 mod 11
            1.to_biguint().unwrap(),
            // g^a_1 = 3^3 = 27 mod 11
            5.to_biguint().unwrap(),
            // g^a_2 = 3^4 = 81 mod 11 = 4 mod 11
            4.to_biguint().unwrap(),
        ];
        let shares = vss::generate_shares(&a, 5, &q);

        for (i, s_i) in shares {
            assert!(
                vss::verify_share(&i, &s_i, &g, &c, &p),
                "failed {} {}",
                i,
                s_i
            );
        }
    }

    #[test]
    fn reconstruct() {
        let shares = vec![
            (2.to_biguint().unwrap(), 1942.to_biguint().unwrap()),
            (4.to_biguint().unwrap(), 3402.to_biguint().unwrap()),
            (5.to_biguint().unwrap(), 4414.to_biguint().unwrap()),
        ];
        // random prime
        let q = 13931.to_biguint().unwrap();

        assert_eq!(
            1234 as usize,
            vss::reconstruct(&shares, &q).to_usize().unwrap()
        );
    }

    #[test]
    fn generate_commitments() {
        let a = vec![
            3.to_biguint().unwrap(),
            5.to_biguint().unwrap(),
            8.to_biguint().unwrap(),
        ];
        let p = 11.to_biguint().unwrap();
        let g = 3.to_biguint().unwrap();

        let expected = vec![
            // 3^(3) = 27 mod 11 = 5 mod 11
            5.to_biguint().unwrap(),
            // 3^(5) = 243 mod 11 = 1 mod 11
            1.to_biguint().unwrap(),
            // 3^(8) = 6561 mod 11 = 5 mod 11
            5.to_biguint().unwrap(),
        ];
        let actual = vss::generate_commitments(&a, &g, &p);

        assert_eq!(expected, actual);
    }
}
