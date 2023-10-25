# Feldman's Verifiable Secret Sharing
exercise to implement https://en.wikipedia.org/wiki/Verifiable_secret_sharing

A secret sharing scheme allows a secret to be shared securely by breaking into pieces. A threshold of
pieces can be combined together to later reconstruct the secret. In Shamir's secret sharing protocal,
this is done by using the fact that one can reconstruct a unique polynomial of degree n given n + 1 unique points
(LaGrange Inteprolating Polynomials). 

A dealer that wants a 3/5 threshold (meaning 5 hold shares and a minimum 3 is needed to reconstruct the secret)
will construct a polynomial of degree 2 with random coefficients and the y-intercept being the secret value and generate 5 shares. 3 shares are
needed to generate the correct polynomials and secret and then interpret the polynomial at 0 so the secret can be recovered as the secret is the y-intercept.

A verifiable secret sharing adds an additional step of generating "commitments" when a dealer generates a share so players can verify if their
share is correct and other shares given to them are correct as well. This can protect against a malicious dealer.

The sharing process must generate two primes p, q s.t. q | p - 1. Polynomials
construction, interpretation, and generating shares all happen over primefield q.
While commitment generation and verification is done over primefield p. We pick these primes, so a generator g of order q over primefield p can be constructed.
This generator g of order q over p has the property such that for every n coprime to p there is a power k of g that is congruent to n
modulo p. This k is also called discrete logairthm of a base g over p. This is used in generating commitments as solving discrete logarithms
is generally known to be hard. So commitments, c_i, are generated for every coefficient a1...an by using the generator g, g^a1...g^an mod p.
A share, v, can be verified if g^v mod p = product of of c_0,c_1^(i^1),c_2^(i^2),...,c_n^(i^n) mod p.

In `vss.rs` the secret sharing algorithm is defined. In `dealer.rss` the sharing phase is defined and `player.rs` the reconstruct phase is defined. 