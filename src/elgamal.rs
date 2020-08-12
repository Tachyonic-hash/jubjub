use crate::{ExtendedPoint, Fr};

use core::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

/// Tuple for assymetric encryption using ElGamal algorithm.
///
/// ## Example
///
/// ```rust
/// use dusk_jubjub::elgamal::ElgamalCipher;
/// use dusk_jubjub::{Fr, GENERATOR_EXTENDED};
///
/// fn main() {
///     // Bob's (sender) secret and message
///     let bob_secret = Fr::random(&mut rand::thread_rng());
///     let message = Fr::random(&mut rand::thread_rng());
///     let message = GENERATOR_EXTENDED * message;
///
///     // Alice's (receiver) secret and public
///     let alice_secret = Fr::random(&mut rand::thread_rng());
///     let alice_public = GENERATOR_EXTENDED * alice_secret;
///
///     let cipher = ElgamalCipher::encrypt(
///         &bob_secret,
///         &alice_public,
///         &GENERATOR_EXTENDED,
///         &message,
///     );
///     let decrypt = cipher.decrypt(&alice_secret);
///
///     assert_eq!(message, decrypt);
/// }
/// ```
///
/// 1. Let `p` and `G = α` be defined by the parameters of JubJub.
/// 2. Let `a` be Alice's secret, and `A = G · a`
/// 3. Let `b` be Bob's secret, and `B = G · b`
///
/// #### Encryption
/// Bob should do the following:
///
/// 1. Obtain Alice’s authentic public key `A`.
/// 2. Represent the message `M` as a point of JubJub defined such as `M = G · m` where `m` is a scalar in `Fr`.
/// 3. Compute `γ = G · b` and `δ = M + (A · b)`.
/// 4. Send the ciphertext `c = (γ; δ)` to Alice.
///
/// #### Decryption
/// To recover plaintext `M` from `c`, Alice should do the following:
///
/// 1. Recover `M` by computing `δ - γ · a`.
///
/// #### Homomorphism
/// A function `f` is homomorphic when `f(a · b) = f(a) · f(b)`.
///
/// This implementation extends the homomorphic property of ElGamal to addition, subtraction and
/// multiplication.
///
/// The addition and subtraction are homomorphic with other [`ElgamalCipher`] structures.
///
/// The multiplication is homomorphic with [`Fr`] scalars.
///
/// Being `E` the encrypt and `D` the decrypt functions, here follows an example:
/// `D[x * E(a + b)] == D{x * [E(a) + E(b)]}`
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct ElgamalCipher {
    gamma: ExtendedPoint,
    delta: ExtendedPoint,
}

impl ElgamalCipher {
    /// [`ElgamalCipher`] constructor
    pub fn new(gamma: ExtendedPoint, delta: ExtendedPoint) -> Self {
        Self { gamma, delta }
    }

    /// Getter for the gamma public key
    pub fn gamma(&self) -> &ExtendedPoint {
        &self.gamma
    }

    /// Getter for the delta ciphertext
    pub fn delta(&self) -> &ExtendedPoint {
        &self.delta
    }

    /// Uses assymetric encryption to return a cipher construction.
    ///
    /// The decryption will expect the secret of `public`.
    pub fn encrypt(
        secret: &Fr,
        public: &ExtendedPoint,
        generator: &ExtendedPoint,
        message: &ExtendedPoint,
    ) -> Self {
        let gamma = generator * secret;
        let delta = message + public * secret;

        Self::new(gamma, delta)
    }

    /// Perform the decryption with the provided secret.
    pub fn decrypt(&self, secret: &Fr) -> ExtendedPoint {
        self.delta - self.gamma * secret
    }
}

impl Add for &ElgamalCipher {
    type Output = ElgamalCipher;

    fn add(self, other: &ElgamalCipher) -> ElgamalCipher {
        ElgamalCipher::new(self.gamma + other.gamma, self.delta + other.delta)
    }
}

impl Add for ElgamalCipher {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        &self + &other
    }
}

impl AddAssign for ElgamalCipher {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for &ElgamalCipher {
    type Output = ElgamalCipher;

    fn sub(self, other: &ElgamalCipher) -> ElgamalCipher {
        ElgamalCipher::new(self.gamma - other.gamma, self.delta - other.delta)
    }
}

impl Sub for ElgamalCipher {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        &self - &other
    }
}

impl SubAssign for ElgamalCipher {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul<&Fr> for &ElgamalCipher {
    type Output = ElgamalCipher;

    fn mul(self, rhs: &Fr) -> ElgamalCipher {
        ElgamalCipher::new(self.gamma * rhs, self.delta * rhs)
    }
}

impl Mul<Fr> for &ElgamalCipher {
    type Output = ElgamalCipher;

    fn mul(self, rhs: Fr) -> ElgamalCipher {
        self * &rhs
    }
}

impl MulAssign<Fr> for ElgamalCipher {
    fn mul_assign(&mut self, rhs: Fr) {
        *self = &*self * &rhs;
    }
}

impl<'b> MulAssign<&'b Fr> for ElgamalCipher {
    fn mul_assign(&mut self, rhs: &'b Fr) {
        *self = &*self * rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::ElgamalCipher;
    use crate::{ExtendedPoint, Fr, GENERATOR_EXTENDED};

    fn gen() -> (Fr, ExtendedPoint, Fr, ExtendedPoint) {
        let a = Fr::random(&mut rand::thread_rng());
        let a_g = GENERATOR_EXTENDED * a;

        let b = Fr::random(&mut rand::thread_rng());
        let b_g = GENERATOR_EXTENDED * b;

        (a, a_g, b, b_g)
    }

    #[test]
    fn encrypt() {
        let (a, _, b, b_g) = gen();

        let m = Fr::random(&mut rand::thread_rng());
        let m = GENERATOR_EXTENDED * m;

        let cipher = ElgamalCipher::encrypt(&a, &b_g, &GENERATOR_EXTENDED, &m);
        let decrypt = cipher.decrypt(&b);

        assert_eq!(m, decrypt);
    }

    #[test]
    fn wrong_key() {
        let (a, _, b, b_g) = gen();

        let m = Fr::random(&mut rand::thread_rng());
        let m = GENERATOR_EXTENDED * m;

        let cipher = ElgamalCipher::encrypt(&a, &b_g, &GENERATOR_EXTENDED, &m);

        let wrong = b - Fr::one();
        let decrypt = cipher.decrypt(&wrong);

        assert_ne!(m, decrypt);
    }

    #[test]
    fn homomorphic_add() {
        let (a, _, b, b_g) = gen();

        let mut m = [Fr::zero(); 4];
        m.iter_mut()
            .for_each(|x| *x = Fr::random(&mut rand::thread_rng()));

        let mut m_g = [ExtendedPoint::default(); 4];
        m_g.iter_mut()
            .zip(m.iter())
            .for_each(|(x, y)| *x = GENERATOR_EXTENDED * y);

        let result = m[0] + m[1] + m[2] + m[3];
        let result = GENERATOR_EXTENDED * result;

        let mut cipher = [ElgamalCipher::default(); 4];
        cipher.iter_mut().zip(m_g.iter()).for_each(|(x, y)| {
            *x = ElgamalCipher::encrypt(&a, &b_g, &GENERATOR_EXTENDED, y)
        });

        let mut hom_cipher = cipher[0] + cipher[1];
        hom_cipher += cipher[2];
        hom_cipher = &hom_cipher + &cipher[3];

        let hom_decrypt = hom_cipher.decrypt(&b);

        assert_eq!(result, hom_decrypt);
    }

    #[test]
    fn homomorphic_sub() {
        let (a, _, b, b_g) = gen();

        let mut m = [Fr::zero(); 4];
        m.iter_mut()
            .for_each(|x| *x = Fr::random(&mut rand::thread_rng()));

        let mut m_g = [ExtendedPoint::default(); 4];
        m_g.iter_mut()
            .zip(m.iter())
            .for_each(|(x, y)| *x = GENERATOR_EXTENDED * y);

        let result = m[0] - m[1] - m[2] - m[3];
        let result = GENERATOR_EXTENDED * result;

        let mut cipher = [ElgamalCipher::default(); 4];
        cipher.iter_mut().zip(m_g.iter()).for_each(|(x, y)| {
            *x = ElgamalCipher::encrypt(&a, &b_g, &GENERATOR_EXTENDED, y)
        });

        let mut hom_cipher = cipher[0] - cipher[1];
        hom_cipher -= cipher[2];
        hom_cipher = &hom_cipher - &cipher[3];

        let hom_decrypt = hom_cipher.decrypt(&b);

        assert_eq!(result, hom_decrypt);
    }

    #[test]
    fn homomorphic_mul() {
        let (a, _, b, b_g) = gen();

        let mut m = [Fr::zero(); 4];
        m.iter_mut()
            .for_each(|x| *x = Fr::random(&mut rand::thread_rng()));

        let mut m_g = [ExtendedPoint::default(); 4];
        m_g.iter_mut()
            .zip(m.iter())
            .for_each(|(x, y)| *x = GENERATOR_EXTENDED * y);

        let result = m[0] * m[1] * m[2] * m[3];
        let result = GENERATOR_EXTENDED * result;

        let mut cipher =
            ElgamalCipher::encrypt(&a, &b_g, &GENERATOR_EXTENDED, &m_g[0]);

        cipher = &cipher * &m[1];
        cipher = &cipher * m[2];
        cipher *= m[3];

        let decrypt = cipher.decrypt(&b);

        assert_eq!(result, decrypt);
    }
}
