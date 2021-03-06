// Copyright (C) 2019-2020 Aleo Systems Inc.
// This file is part of the snarkOS library.

// The snarkOS library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkOS library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkOS library. If not, see <https://www.gnu.org/licenses/>.

use crate::{impl_bytes, *};
use core::ops::{Add, AddAssign};
use snarkos_errors::serialization::SerializationError;
use snarkos_models::curves::{AffineCurve, PairingCurve, PairingEngine, PrimeField, ProjectiveCurve, Zero};
use snarkos_utilities::{
    bytes::ToBytes,
    error,
    serialize::{CanonicalDeserialize, CanonicalSerialize},
};
use std::collections::BTreeMap;

/// `UniversalParams` are the universal parameters for the KZG10 scheme.
#[derive(Derivative)]
#[derivative(Default(bound = ""), Clone(bound = ""), Debug(bound = ""))]
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct UniversalParams<E: PairingEngine> {
    /// Group elements of the form `{ \beta^i G }`, where `i` ranges from 0 to `degree`.
    pub powers_of_g: Vec<E::G1Affine>,
    /// Group elements of the form `{ \beta^i \gamma G }`, where `i` ranges from 0 to `degree`.
    pub powers_of_gamma_g: BTreeMap<usize, E::G1Affine>,
    /// The generator of G2.
    pub h: E::G2Affine,
    /// \beta times the above generator of G2.
    pub beta_h: E::G2Affine,
    /// Group elements of the form `{ \beta^i G2 }`, where `i` ranges from `0` to `-degree`.
    pub prepared_neg_powers_of_h: BTreeMap<usize, <E::G2Affine as PairingCurve>::Prepared>,
    /// The generator of G2, prepared for use in pairings.
    #[derivative(Debug = "ignore")]
    pub prepared_h: <E::G2Affine as PairingCurve>::Prepared,
    /// \beta times the above generator of G2, prepared for use in pairings.
    #[derivative(Debug = "ignore")]
    pub prepared_beta_h: <E::G2Affine as PairingCurve>::Prepared,
}

impl_bytes!(UniversalParams);

impl<E: PairingEngine> PCUniversalParams for UniversalParams<E> {
    fn max_degree(&self) -> usize {
        self.powers_of_g.len() - 1
    }
}

/// `Powers` is used to commit to and create evaluation proofs for a given
/// polynomial.
#[derive(Derivative)]
#[derivative(Default(bound = ""), Hash(bound = ""), Clone(bound = ""), Debug(bound = ""))]
pub struct Powers<'a, E: PairingEngine> {
    /// Group elements of the form `β^i G`, for different values of `i`.
    pub powers_of_g: Cow<'a, [E::G1Affine]>,
    /// Group elements of the form `β^i γG`, for different values of `i`.
    pub powers_of_gamma_g: Cow<'a, [E::G1Affine]>,
}

impl<E: PairingEngine> Powers<'_, E> {
    /// The number of powers in `self`.
    pub fn size(&self) -> usize {
        self.powers_of_g.len()
    }
}

/// `VerifierKey` is used to check evaluation proofs for a given commitment.
#[derive(Derivative)]
#[derivative(Default(bound = ""), Clone(bound = ""), Debug(bound = ""))]
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct VerifierKey<E: PairingEngine> {
    /// The generator of G1.
    pub g: E::G1Affine,
    /// The generator of G1 that is used for making a commitment hiding.
    pub gamma_g: E::G1Affine,
    /// The generator of G2.
    pub h: E::G2Affine,
    /// \beta times the above generator of G2.
    pub beta_h: E::G2Affine,
    /// The generator of G2, prepared for use in pairings.
    #[derivative(Debug = "ignore")]
    pub prepared_h: <E::G2Affine as PairingCurve>::Prepared,
    /// \beta times the above generator of G2, prepared for use in pairings.
    #[derivative(Debug = "ignore")]
    pub prepared_beta_h: <E::G2Affine as PairingCurve>::Prepared,
}
impl_bytes!(VerifierKey);

/// `Commitment` commits to a polynomial. It is output by `KZG10::commit`.
#[derive(Derivative)]
#[derivative(
    Default(bound = ""),
    Hash(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct Commitment<E: PairingEngine>(
    /// The commitment is a group element.
    pub E::G1Affine,
);

impl_bytes!(Commitment);

impl<E: PairingEngine> PCCommitment for Commitment<E> {
    #[inline]
    fn empty() -> Self {
        Commitment(E::G1Affine::zero())
    }

    fn has_degree_bound(&self) -> bool {
        false
    }

    fn is_in_correct_subgroup_assuming_on_curve(&self) -> bool {
        self.0.is_in_correct_subgroup_assuming_on_curve()
    }
}

impl<'a, E: PairingEngine> AddAssign<(E::Fr, &'a Commitment<E>)> for Commitment<E> {
    #[inline]
    fn add_assign(&mut self, (f, other): (E::Fr, &'a Commitment<E>)) {
        let mut other = other.0.mul(f.into_repr());
        other.add_assign_mixed(&self.0);
        self.0 = other.into();
    }
}

/// `Randomness` hides the polynomial inside a commitment. It is output by `KZG10::commit`.
#[derive(Derivative)]
#[derivative(
    Default(bound = ""),
    Hash(bound = ""),
    Clone(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct Randomness<E: PairingEngine> {
    /// For KZG10, the commitment randomness is a random polynomial.
    pub blinding_polynomial: Polynomial<E::Fr>,
}
impl_bytes!(Randomness);

impl<E: PairingEngine> Randomness<E> {
    /// Does `self` provide any hiding properties to the corresponding commitment?
    /// `self.is_hiding() == true` only if the underlying polynomial is non-zero.
    #[inline]
    pub fn is_hiding(&self) -> bool {
        !self.blinding_polynomial.is_zero()
    }

    /// What is the degree of the hiding polynomial for a given hiding bound?
    #[inline]
    pub fn calculate_hiding_polynomial_degree(hiding_bound: usize) -> usize {
        hiding_bound + 1
    }
}

impl<E: PairingEngine> PCRandomness for Randomness<E> {
    fn empty() -> Self {
        Self {
            blinding_polynomial: Polynomial::zero(),
        }
    }

    fn rand<R: RngCore>(hiding_bound: usize, _: bool, rng: &mut R) -> Self {
        let mut randomness = Randomness::empty();
        let hiding_poly_degree = Self::calculate_hiding_polynomial_degree(hiding_bound);
        randomness.blinding_polynomial = Polynomial::rand(hiding_poly_degree, rng);
        randomness
    }
}

impl<'a, E: PairingEngine> Add<&'a Randomness<E>> for Randomness<E> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: &'a Self) -> Self {
        self.blinding_polynomial += &other.blinding_polynomial;
        self
    }
}

impl<'a, E: PairingEngine> Add<(E::Fr, &'a Randomness<E>)> for Randomness<E> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: (E::Fr, &'a Randomness<E>)) -> Self {
        self += other;
        self
    }
}

impl<'a, E: PairingEngine> AddAssign<&'a Randomness<E>> for Randomness<E> {
    #[inline]
    fn add_assign(&mut self, other: &'a Self) {
        self.blinding_polynomial += &other.blinding_polynomial;
    }
}

impl<'a, E: PairingEngine> AddAssign<(E::Fr, &'a Randomness<E>)> for Randomness<E> {
    #[inline]
    fn add_assign(&mut self, (f, other): (E::Fr, &'a Randomness<E>)) {
        self.blinding_polynomial += (f, &other.blinding_polynomial);
    }
}

/// `Proof` is an evaluation proof that is output by `KZG10::open`.
#[derive(Derivative)]
#[derivative(
    Default(bound = ""),
    Hash(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct Proof<E: PairingEngine> {
    /// This is a commitment to the witness polynomial; see [KZG10] for more details.
    pub w: E::G1Affine,
    /// This is the evaluation of the random polynomial at the point for which
    /// the evaluation proof was produced.
    pub random_v: Option<E::Fr>,
}
impl_bytes!(Proof);

impl<E: PairingEngine> PCProof for Proof<E> {}
