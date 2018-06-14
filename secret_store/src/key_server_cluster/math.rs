// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use ethkey::{Public, Secret, Signature, Random, Generator, math};
use ethereum_types::{H256, U256};
use hash::keccak;
use key_server_cluster::Error;

/// Encryption result.
#[derive(Debug)]
pub struct EncryptedSecret {
	/// Common encryption point.
	pub common_point: Public,
	/// Ecnrypted point.
	pub encrypted_point: Public,
}

/// Create zero scalar.
pub fn zero_scalar() -> Secret {
	Secret::zero()
}

/// Convert hash to EC scalar (modulo curve order).
pub fn to_scalar(hash: H256) -> Result<Secret, Error> {
	let scalar: U256 = hash.into();
	let scalar: H256 = (scalar % math::curve_order()).into();
	let scalar = Secret::from(scalar.0);
	scalar.check_validity()?;
	Ok(scalar)
}

/// Generate random scalar.
pub fn generate_random_scalar() -> Result<Secret, Error> {
	Ok(Random.generate()?.secret().clone())
}

/// Generate random point.
pub fn generate_random_point() -> Result<Public, Error> {
	Ok(Random.generate()?.public().clone())
}

/// Get X coordinate of point.
fn public_x(public: &Public) -> H256 {
	public[0..32].into()
}

/// Get Y coordinate of point.
fn public_y(public: &Public) -> H256 {
	public[32..64].into()
}

/// Compute publics sum.
pub fn compute_public_sum<'a, I>(mut publics: I) -> Result<Public, Error> where I: Iterator<Item=&'a Public> {
	let mut sum = publics.next().expect("compute_public_sum is called when there's at least one public; qed").clone();
	while let Some(public) = publics.next() {
		math::public_add(&mut sum, &public)?;
	}
	Ok(sum)
}

/// Compute secrets sum.
pub fn compute_secret_sum<'a, I>(mut secrets: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	let mut sum = secrets.next().expect("compute_secret_sum is called when there's at least one secret; qed").clone();
	while let Some(secret) = secrets.next() {
		sum.add(secret)?;
	}
	Ok(sum)
}

/// Compute secrets multiplication.
pub fn compute_secret_mul(secret1: &Secret, secret2: &Secret) -> Result<Secret, Error> {
	let mut secret_mul = secret1.clone();
	secret_mul.mul(secret2)?;
	Ok(secret_mul)
}

/// Compute secrets 'shadow' multiplication: coeff * multiplication(s[j] / (s[i] - s[j])) for every i != j
pub fn compute_shadow_mul<'a, I>(coeff: &Secret, self_secret: &Secret, mut other_secrets: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	// when there are no other secrets, only coeff is left
	let other_secret = match other_secrets.next() {
		Some(other_secret) => other_secret,
		None => return Ok(coeff.clone()),
	};

	let mut shadow_mul = self_secret.clone();
	shadow_mul.sub(other_secret)?;
	shadow_mul.inv()?;
	shadow_mul.mul(other_secret)?;
	while let Some(other_secret) = other_secrets.next() {
		let mut shadow_mul_element = self_secret.clone();
		shadow_mul_element.sub(other_secret)?;
		shadow_mul_element.inv()?;
		shadow_mul_element.mul(other_secret)?;
		shadow_mul.mul(&shadow_mul_element)?;
	}

	shadow_mul.mul(coeff)?;
	Ok(shadow_mul)
}

/// Update point by multiplying to random scalar
pub fn update_random_point(point: &mut Public) -> Result<(), Error> {
	Ok(math::public_mul_secret(point, &generate_random_scalar()?)?)
}

/// Generate random polynom of threshold degree
pub fn generate_random_polynom(threshold: usize) -> Result<Vec<Secret>, Error> {
	(0..threshold + 1)
		.map(|_| generate_random_scalar())
		.collect()
}

/// Compute value of polynom, using `node_number` as argument
pub fn compute_polynom(polynom: &[Secret], node_number: &Secret) -> Result<Secret, Error> {
	debug_assert!(!polynom.is_empty());

	let mut result = polynom[0].clone();
	for i in 1..polynom.len() {
		// calculate pow(node_number, i)
		let mut appendum = node_number.clone();
		appendum.pow(i)?;

		// calculate coeff * pow(point, i)
		appendum.mul(&polynom[i])?;

		// calculate result + coeff * pow(point, i)
		result.add(&appendum)?;
	}

	Ok(result)
}

/// Generate public keys for other participants.
pub fn public_values_generation(threshold: usize, derived_point: &Public, polynom1: &[Secret], polynom2: &[Secret]) -> Result<Vec<Public>, Error> {
	debug_assert_eq!(polynom1.len(), threshold + 1);
	debug_assert_eq!(polynom2.len(), threshold + 1);

	// compute t+1 public values
	let mut publics = Vec::with_capacity(threshold + 1);
	for i in 0..threshold + 1 {
		let coeff1 = &polynom1[i];

		let mut multiplication1 = math::generation_point();
		math::public_mul_secret(&mut multiplication1, &coeff1)?;

		let coeff2 = &polynom2[i];
		let mut multiplication2 = derived_point.clone();
		math::public_mul_secret(&mut multiplication2, &coeff2)?;

		math::public_add(&mut multiplication1, &multiplication2)?;

		publics.push(multiplication1);
	}
	debug_assert_eq!(publics.len(), threshold + 1);

	Ok(publics)
}

/// Check keys passed by other participants.
pub fn keys_verification(threshold: usize, derived_point: &Public, number_id: &Secret, secret1: &Secret, secret2: &Secret, publics: &[Public]) -> Result<bool, Error> {
	// calculate left part
	let mut multiplication1 = math::generation_point();
	math::public_mul_secret(&mut multiplication1, secret1)?;

	let mut multiplication2 = derived_point.clone();
	math::public_mul_secret(&mut multiplication2, secret2)?;

	math::public_add(&mut multiplication1, &multiplication2)?;
	let left = multiplication1;

	// calculate right part
	let mut right = publics[0].clone();
	for i in 1..threshold + 1 {
		let mut secret_pow = number_id.clone();
		secret_pow.pow(i)?;

		let mut public_k = publics[i].clone();
		math::public_mul_secret(&mut public_k, &secret_pow)?;

		math::public_add(&mut right, &public_k)?;
	}

	Ok(left == right)
}

/// Compute secret subshare from passed secret value.
pub fn compute_secret_subshare<'a, I>(threshold: usize, secret_value: &Secret, sender_id_number: &Secret, other_id_numbers: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	let mut subshare = compute_shadow_mul(secret_value, sender_id_number, other_id_numbers)?;
	if threshold % 2 != 0 {
		subshare.neg()?;
	}

	Ok(subshare)
}

/// Compute secret share.
pub fn compute_secret_share<'a, I>(secret_values: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	compute_secret_sum(secret_values)
}

/// Compute public key share.
pub fn compute_public_share(self_secret_value: &Secret) -> Result<Public, Error> {
	let mut public_share = math::generation_point();
	math::public_mul_secret(&mut public_share, self_secret_value)?;
	Ok(public_share)
}

/// Compute joint public key.
pub fn compute_joint_public<'a, I>(public_shares: I) -> Result<Public, Error> where I: Iterator<Item=&'a Public> {
	compute_public_sum(public_shares)
}

/// Compute joint secret key from N secret coefficients.
#[cfg(test)]
pub fn compute_joint_secret<'a, I>(secret_coeffs: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	compute_secret_sum(secret_coeffs)
}

/// Compute joint secret key from t+1 secret shares.
pub fn compute_joint_secret_from_shares<'a>(t: usize, secret_shares: &[&'a Secret], id_numbers: &[&'a Secret]) -> Result<Secret, Error> {
	let secret_share_0 = secret_shares[0];
	let id_number_0 = id_numbers[0];
	let other_nodes_numbers = id_numbers.iter().skip(1).cloned();
	let mut result = compute_node_shadow(secret_share_0, id_number_0, other_nodes_numbers)?;
	for i in 1..secret_shares.len() {
		let secret_share_i = secret_shares[i];
		let id_number_i = id_numbers[i];
		let other_nodes_numbers = id_numbers.iter().enumerate().filter(|&(j, _)| j != i).map(|(_, n)| n).cloned();
		let addendum = compute_node_shadow(secret_share_i, id_number_i, other_nodes_numbers)?;
		result.add(&addendum)?;
	}

	if t % 2 != 0 {
		result.neg()?;
	}

	Ok(result)
}

/// Encrypt secret with joint public key.
pub fn encrypt_secret(secret: &Public, joint_public: &Public) -> Result<EncryptedSecret, Error> {
	// this is performed by KS-cluster client (or KS master)
	let key_pair = Random.generate()?;

	// k * T
	let mut common_point = math::generation_point();
	math::public_mul_secret(&mut common_point, key_pair.secret())?;

	// M + k * y
	let mut encrypted_point = joint_public.clone();
	math::public_mul_secret(&mut encrypted_point, key_pair.secret())?;
	math::public_add(&mut encrypted_point, secret)?;

	Ok(EncryptedSecret {
		common_point: common_point,
		encrypted_point: encrypted_point,
	})
}

/// Compute shadow for the node.
pub fn compute_node_shadow<'a, I>(node_secret_share: &Secret, node_number: &Secret, other_nodes_numbers: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	compute_shadow_mul(node_secret_share, node_number, other_nodes_numbers)
}

/// Compute shadow point for the node.
pub fn compute_node_shadow_point(access_key: &Secret, common_point: &Public, node_shadow: &Secret, decrypt_shadow: Option<Secret>) -> Result<(Public, Option<Secret>), Error> {
	let mut shadow_key = node_shadow.clone();
	let decrypt_shadow = match decrypt_shadow {
		None => None,
		Some(mut decrypt_shadow) => {
			// update shadow key
			shadow_key.mul(&decrypt_shadow)?;
			// now udate decrypt shadow itself
			decrypt_shadow.dec()?;
			decrypt_shadow.mul(node_shadow)?;
			Some(decrypt_shadow)
		}
	};
	shadow_key.mul(access_key)?;

	let mut node_shadow_point = common_point.clone();
	math::public_mul_secret(&mut node_shadow_point, &shadow_key)?;
	Ok((node_shadow_point, decrypt_shadow))
}

/// Compute joint shadow point.
pub fn compute_joint_shadow_point<'a, I>(nodes_shadow_points: I) -> Result<Public, Error> where I: Iterator<Item=&'a Public> {
	compute_public_sum(nodes_shadow_points)
}

/// Compute joint shadow point (version for tests).
#[cfg(test)]
pub fn compute_joint_shadow_point_test<'a, I>(access_key: &Secret, common_point: &Public, nodes_shadows: I) -> Result<Public, Error> where I: Iterator<Item=&'a Secret> {
	let mut joint_shadow = compute_secret_sum(nodes_shadows)?;
	joint_shadow.mul(access_key)?;

	let mut joint_shadow_point = common_point.clone();
	math::public_mul_secret(&mut joint_shadow_point, &joint_shadow)?;
	Ok(joint_shadow_point)
}

/// Decrypt data using joint shadow point.
pub fn decrypt_with_joint_shadow(threshold: usize, access_key: &Secret, encrypted_point: &Public, joint_shadow_point: &Public) -> Result<Public, Error> {
	let mut inv_access_key = access_key.clone();
	inv_access_key.inv()?;

	let mut mul = joint_shadow_point.clone();
	math::public_mul_secret(&mut mul, &inv_access_key)?;

	let mut decrypted_point = encrypted_point.clone();
	if threshold % 2 != 0 {
		math::public_add(&mut decrypted_point, &mul)?;
	} else {
		math::public_sub(&mut decrypted_point, &mul)?;
	}

	Ok(decrypted_point)
}

/// Prepare common point for shadow decryption.
pub fn make_common_shadow_point(threshold: usize, mut common_point: Public) -> Result<Public, Error> {
	if threshold % 2 != 1 {
		Ok(common_point)
	} else {
		math::public_negate(&mut common_point)?;
		Ok(common_point)
	}
}

/// Decrypt shadow-encrypted secret.
#[cfg(test)]
pub fn decrypt_with_shadow_coefficients(mut decrypted_shadow: Public, mut common_shadow_point: Public, shadow_coefficients: Vec<Secret>) -> Result<Public, Error> {
	let shadow_coefficients_sum = compute_secret_sum(shadow_coefficients.iter())?;
	math::public_mul_secret(&mut common_shadow_point, &shadow_coefficients_sum)?;
	math::public_add(&mut decrypted_shadow, &common_shadow_point)?;
	Ok(decrypted_shadow)
}

/// Decrypt data using joint secret (version for tests).
#[cfg(test)]
pub fn decrypt_with_joint_secret(encrypted_point: &Public, common_point: &Public, joint_secret: &Secret) -> Result<Public, Error> {
	let mut common_point_mul = common_point.clone();
	math::public_mul_secret(&mut common_point_mul, joint_secret)?;

	let mut decrypted_point = encrypted_point.clone();
	math::public_sub(&mut decrypted_point, &common_point_mul)?;

	Ok(decrypted_point)
}

/// Combine message hash with public key X coordinate.
pub fn combine_message_hash_with_public(message_hash: &H256, public: &Public) -> Result<Secret, Error> {
	// buffer is just [message_hash | public.x]
	let mut buffer = [0; 64];
	buffer[0..32].copy_from_slice(&message_hash[0..32]);
	buffer[32..64].copy_from_slice(&public[0..32]);

	// calculate hash of buffer
	let hash = keccak(&buffer[..]);

	// map hash to EC finite field value
	to_scalar(hash)
}

/// Compute Schnorr signature share.
pub fn compute_schnorr_signature_share<'a, I>(threshold: usize, combined_hash: &Secret, one_time_secret_coeff: &Secret, node_secret_share: &Secret, node_number: &Secret, other_nodes_numbers: I)
	-> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	let mut sum = one_time_secret_coeff.clone();
	let mut subtrahend = compute_shadow_mul(combined_hash, node_number, other_nodes_numbers)?;
	subtrahend.mul(node_secret_share)?;
	if threshold % 2 == 0 {
		sum.sub(&subtrahend)?;
	} else {
		sum.add(&subtrahend)?;
	}
	Ok(sum)
}

/// Check Schnorr signature share.
pub fn _check_schnorr_signature_share<'a, I>(_combined_hash: &Secret, _signature_share: &Secret, _public_share: &Public, _one_time_public_share: &Public, _node_numbers: I)
	-> Result<bool, Error> where I: Iterator<Item=&'a Secret> {
	// TODO [Trust]: in paper partial signature is checked using comparison:
	//    sig[i] * T                                  = r[i] - c * lagrange_coeff(i) * y[i]
	// => (k[i] - c * lagrange_coeff(i) * s[i]) * T   = r[i] - c * lagrange_coeff(i) * y[i]
	// => k[i] * T - c * lagrange_coeff(i) * s[i] * T = k[i] * T - c * lagrange_coeff(i) * y[i]
	// => this means that y[i] = s[i] * T
	// but when verifying signature (for t = 1), nonce public (r) is restored using following expression:
	// r = (sig[0] + sig[1]) * T - c * y
	// r = (k[0] - c * lagrange_coeff(0) * s[0] + k[1] - c * lagrange_coeff(1) * s[1]) * T - c * y
	// r = (k[0] + k[1]) * T - c * (lagrange_coeff(0) * s[0] + lagrange_coeff(1) * s[1]) * T - c * y
	// r = r - c * (lagrange_coeff(0) * s[0] + lagrange_coeff(1) * s[1]) * T - c * y
	// => -c * y = c * (lagrange_coeff(0) * s[0] + lagrange_coeff(1) * s[1]) * T
	// => -y = (lagrange_coeff(0) * s[0] + lagrange_coeff(1) * s[1]) * T
	// => y[i] != s[i] * T
	// => some other way is required
	Ok(true)
}

/// Compute Schnorr signature.
pub fn compute_schnorr_signature<'a, I>(signature_shares: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	compute_secret_sum(signature_shares)
}

/// Locally compute Schnorr signature as described in https://en.wikipedia.org/wiki/Schnorr_signature#Signing.
#[cfg(test)]
pub fn local_compute_schnorr_signature(nonce: &Secret, secret: &Secret, message_hash: &Secret) -> Result<(Secret, Secret), Error> {
	let mut nonce_public = math::generation_point();
	math::public_mul_secret(&mut nonce_public, &nonce).unwrap();

	let combined_hash = combine_message_hash_with_public(message_hash, &nonce_public)?;

	let mut sig_subtrahend = combined_hash.clone();
	sig_subtrahend.mul(secret)?;
	let mut sig = nonce.clone();
	sig.sub(&sig_subtrahend)?;

	Ok((combined_hash, sig))
}

/// Verify Schnorr signature as described in https://en.wikipedia.org/wiki/Schnorr_signature#Verifying.
#[cfg(test)]
pub fn verify_schnorr_signature(public: &Public, signature: &(Secret, Secret), message_hash: &H256) -> Result<bool, Error> {
	let mut addendum = math::generation_point();
	math::public_mul_secret(&mut addendum, &signature.1)?;
	let mut nonce_public = public.clone();
	math::public_mul_secret(&mut nonce_public, &signature.0)?;
	math::public_add(&mut nonce_public, &addendum)?;

	let combined_hash = combine_message_hash_with_public(message_hash, &nonce_public)?;
	Ok(combined_hash == signature.0)
}

/// Compute R part of ECDSA signature.
pub fn compute_ecdsa_r(nonce_public: &Public) -> Result<Secret, Error> {
	to_scalar(public_x(nonce_public))
}

/// Compute share of S part of ECDSA signature.
pub fn compute_ecdsa_s_share(inv_nonce_share: &Secret, inv_nonce_mul_secret: &Secret, signature_r: &Secret, message_hash: &Secret) -> Result<Secret, Error> {
	let mut nonce_inv_share_mul_message_hash = inv_nonce_share.clone();
	nonce_inv_share_mul_message_hash.mul(&message_hash.clone().into())?;

	let mut nonce_inv_share_mul_secret_share_mul_r = inv_nonce_mul_secret.clone();
	nonce_inv_share_mul_secret_share_mul_r.mul(signature_r)?;

	let mut signature_s_share = nonce_inv_share_mul_message_hash;
	signature_s_share.add(&nonce_inv_share_mul_secret_share_mul_r)?;

	Ok(signature_s_share)
}

/// Compute S part of ECDSA signature from shares.
pub fn compute_ecdsa_s(t: usize, signature_s_shares: &[Secret], id_numbers: &[Secret]) -> Result<Secret, Error> {
	let double_t = t * 2;
	debug_assert!(id_numbers.len() >= double_t + 1);
	debug_assert_eq!(signature_s_shares.len(), id_numbers.len());

	compute_joint_secret_from_shares(double_t,
		&signature_s_shares.iter().take(double_t + 1).collect::<Vec<_>>(),
		&id_numbers.iter().take(double_t + 1).collect::<Vec<_>>())
}

/// Serialize ECDSA signature to [r][s]v form.
pub fn serialize_ecdsa_signature(nonce_public: &Public, signature_r: Secret, mut signature_s: Secret) -> Signature {
	// compute recovery param
	let mut signature_v = {
		let nonce_public_x = public_x(nonce_public);
		let nonce_public_y: U256 = public_y(nonce_public).into();
		let nonce_public_y_is_odd = !(nonce_public_y % 2.into()).is_zero();
		let bit0 = if nonce_public_y_is_odd { 1u8 } else { 0u8 };
		let bit1 = if nonce_public_x != *signature_r { 2u8 } else { 0u8 };
		bit0 | bit1
	};

	// fix high S
	let curve_order = math::curve_order();
	let curve_order_half = curve_order / 2.into();
	let s_numeric: U256 = (*signature_s).into();
	if s_numeric > curve_order_half {
		let signature_s_hash: H256 = (curve_order - s_numeric).into();
		signature_s = signature_s_hash.into();
		signature_v ^= 1;
	}

	// serialize as [r][s]v
	let mut signature = [0u8; 65];
	signature[..32].copy_from_slice(&**signature_r);
	signature[32..64].copy_from_slice(&**signature_s);
	signature[64] = signature_v;

	signature.into()
}

/// Compute share of ECDSA reversed-nonce coefficient. Result of this_coeff * secret_share gives us a share of inv(nonce).
pub fn compute_ecdsa_inversed_secret_coeff_share(secret_share: &Secret, nonce_share: &Secret, zero_share: &Secret) -> Result<Secret, Error> {
	let mut coeff = secret_share.clone();
	coeff.mul(nonce_share).unwrap();
	coeff.add(zero_share).unwrap();
	Ok(coeff)
}

/// Compute ECDSA reversed-nonce coefficient from its shares. Result of this_coeff * secret_share gives us a share of inv(nonce).
pub fn compute_ecdsa_inversed_secret_coeff_from_shares(t: usize, id_numbers: &[Secret], shares: &[Secret]) -> Result<Secret, Error> {
	debug_assert_eq!(shares.len(), 2 * t + 1);
	debug_assert_eq!(shares.len(), id_numbers.len());

	let u_shares = (0..2*t+1).map(|i| compute_shadow_mul(&shares[i], &id_numbers[i], id_numbers.iter().enumerate()
		.filter(|&(j, _)| i != j)
		.map(|(_, id)| id)
		.take(2 * t))).collect::<Result<Vec<_>, _>>()?;

	// compute u
	let u = compute_secret_sum(u_shares.iter())?;

	// compute inv(u)
	let mut u_inv = u;
	u_inv.inv()?;
	Ok(u_inv)
}

#[cfg(test)]
pub mod tests {
	use std::iter::once;
	use ethkey::{KeyPair, recover, verify_public};
	use super::*;

	#[derive(Clone)]
	struct KeyGenerationArtifacts {
		id_numbers: Vec<Secret>,
		polynoms1: Vec<Vec<Secret>>,
		secrets1: Vec<Vec<Secret>>,
		public_shares: Vec<Public>,
		secret_shares: Vec<Secret>,
		joint_public: Public,
	}

	struct ZeroGenerationArtifacts {
		polynoms1: Vec<Vec<Secret>>,
		secret_shares: Vec<Secret>,
	}

	fn prepare_polynoms1(t: usize, n: usize, secret_required: Option<Secret>) -> Vec<Vec<Secret>> {
		let mut polynoms1: Vec<_> = (0..n).map(|_| generate_random_polynom(t).unwrap()).collect();
		// if we need specific secret to be shared, update polynoms so that sum of their free terms = required secret
		if let Some(mut secret_required) = secret_required {
			for polynom1 in polynoms1.iter_mut().take(n - 1) {
				let secret_coeff1 = generate_random_scalar().unwrap();
				secret_required.sub(&secret_coeff1).unwrap();
				polynom1[0] = secret_coeff1;
			}

			polynoms1[n - 1][0] = secret_required;
		}
		polynoms1
	}

	fn run_key_generation(t: usize, n: usize, id_numbers: Option<Vec<Secret>>, secret_required: Option<Secret>) -> KeyGenerationArtifacts {
		// === PART1: DKG ===

		// data, gathered during initialization
		let derived_point = Random.generate().unwrap().public().clone();
		let id_numbers: Vec<_> = match id_numbers {
			Some(id_numbers) => id_numbers,
			None => (0..n).map(|_| generate_random_scalar().unwrap()).collect(),
		};

		// data, generated during keys dissemination
		let polynoms1 = prepare_polynoms1(t, n, secret_required);
		let secrets1: Vec<_> = (0..n).map(|i| (0..n).map(|j| compute_polynom(&polynoms1[i], &id_numbers[j]).unwrap()).collect::<Vec<_>>()).collect();

		// following data is used only on verification step
		let polynoms2: Vec<_> = (0..n).map(|_| generate_random_polynom(t).unwrap()).collect();
		let secrets2: Vec<_> = (0..n).map(|i| (0..n).map(|j| compute_polynom(&polynoms2[i], &id_numbers[j]).unwrap()).collect::<Vec<_>>()).collect();
		let publics: Vec<_> = (0..n).map(|i| public_values_generation(t, &derived_point, &polynoms1[i], &polynoms2[i]).unwrap()).collect();

		// keys verification
		(0..n).map(|i| (0..n).map(|j| if i != j {
			assert!(keys_verification(t, &derived_point, &id_numbers[i], &secrets1[j][i], &secrets2[j][i], &publics[j]).unwrap());
		}).collect::<Vec<_>>()).collect::<Vec<_>>();

		// data, generated during keys generation
		let public_shares: Vec<_> = (0..n).map(|i| compute_public_share(&polynoms1[i][0]).unwrap()).collect();
		let secret_shares: Vec<_> = (0..n).map(|i| compute_secret_share(secrets1.iter().map(|s| &s[i])).unwrap()).collect();

		// joint public key, as a result of DKG
		let joint_public = compute_joint_public(public_shares.iter()).unwrap();

		KeyGenerationArtifacts {
			id_numbers: id_numbers,
			polynoms1: polynoms1,
			secrets1: secrets1,
			public_shares: public_shares,
			secret_shares: secret_shares,
			joint_public: joint_public,
		}
	}

	fn run_zero_key_generation(t: usize, n: usize, id_numbers: &[Secret]) -> ZeroGenerationArtifacts {
		// data, generated during keys dissemination
		let polynoms1 = prepare_polynoms1(t, n, Some(zero_scalar()));
		let secrets1: Vec<_> = (0..n).map(|i| (0..n).map(|j| compute_polynom(&polynoms1[i], &id_numbers[j]).unwrap()).collect::<Vec<_>>()).collect();

		// data, generated during keys generation
		let secret_shares: Vec<_> = (0..n).map(|i| compute_secret_share(secrets1.iter().map(|s| &s[i])).unwrap()).collect();

		ZeroGenerationArtifacts {
			polynoms1: polynoms1,
			secret_shares: secret_shares,
		}
	}

	fn run_key_share_refreshing(old_t: usize, new_t: usize, new_n: usize, old_artifacts: &KeyGenerationArtifacts) -> KeyGenerationArtifacts {
		// === share refreshing protocol from
		// === based on "Verifiable Secret Redistribution for Threshold Sharing Schemes"
		// === http://www.cs.cmu.edu/~wing/publications/CMU-CS-02-114.pdf

		// generate new id_numbers for new nodes
		let new_nodes = new_n.saturating_sub(old_artifacts.id_numbers.len());
		let id_numbers: Vec<_> = old_artifacts.id_numbers.iter().take(new_n).cloned()
			.chain((0..new_nodes).map(|_| generate_random_scalar().unwrap()))
			.collect();

		// on every authorized node: generate random polynomial ai(j) = si + ... + ai[new_t - 1] * j^(new_t - 1)
		let mut subshare_polynoms = Vec::new();
		for i in 0..old_t+1 {
			let mut subshare_polynom = generate_random_polynom(new_t).unwrap();
			subshare_polynom[0] = old_artifacts.secret_shares[i].clone();
			subshare_polynoms.push(subshare_polynom);
		}

		// on every authorized node: calculate subshare for every new node
		let mut subshares = Vec::new();
		for j in 0..new_n {
			let mut subshares_to_j = Vec::new();
			for i in 0..old_t+1 {
				let subshare_from_i_to_j = compute_polynom(&subshare_polynoms[i], &id_numbers[j]).unwrap();
				subshares_to_j.push(subshare_from_i_to_j);
			}
			subshares.push(subshares_to_j);
		}

		// on every new node: generate new share using Lagrange interpolation
		// on every node: generate new share using Lagrange interpolation
		let mut new_secret_shares = Vec::new();
		for j in 0..new_n {
			let mut subshares_to_j = Vec::new();
			for i in 0..old_t+1 {
				let subshare_from_i = &subshares[j][i];
				let id_number_i = &id_numbers[i];
				let other_id_numbers = (0usize..old_t+1).filter(|j| *j != i).map(|j| &id_numbers[j]);
				let mut subshare_from_i = compute_shadow_mul(subshare_from_i, id_number_i, other_id_numbers).unwrap();
				if old_t % 2 != 0 {
					subshare_from_i.neg().unwrap();
				}
				subshares_to_j.push(subshare_from_i);
			}
			new_secret_shares.push(compute_secret_sum(subshares_to_j.iter()).unwrap());
		}

		let mut result = old_artifacts.clone();
		result.id_numbers = id_numbers;
		result.secret_shares = new_secret_shares;
		result
	}

	fn run_multiplication_protocol(t: usize, secret_shares1: &[Secret], secret_shares2: &[Secret]) -> Vec<Secret> {
		let n = secret_shares1.len();
		assert!(t * 2 + 1 <= n);

		// shares of secrets multiplication = multiplication of secrets shares
		let mul_shares: Vec<_> = (0..n).map(|i| {
			let share1 = secret_shares1[i].clone();
			let share2 = secret_shares2[i].clone();
			let mut mul_share = share1;
			mul_share.mul(&share2).unwrap();
			mul_share
		}).collect();

		mul_shares
	}

	fn run_reciprocal_protocol(t: usize, artifacts: &KeyGenerationArtifacts) -> Vec<Secret> {
		// === Given a secret x mod r which is shared among n players, it is
		// === required to generate shares of inv(x) mod r with out revealing
		// === any information about x or inv(x).
		// === https://www.researchgate.net/publication/280531698_Robust_Threshold_Elliptic_Curve_Digital_Signature

		// generate shared random secret e for given t
		let n = artifacts.id_numbers.len();
		assert!(t * 2 + 1 <= n);
		let e_artifacts = run_key_generation(t, n, Some(artifacts.id_numbers.clone()), None);

		// generate shares of zero for 2 * t threshold
		let z_artifacts = run_zero_key_generation(2 * t, n, &artifacts.id_numbers);

		// each player computes && broadcast u[i] = x[i] * e[i] + z[i]
		let ui: Vec<_> = (0..n).map(|i| compute_ecdsa_inversed_secret_coeff_share(&artifacts.secret_shares[i],
			&e_artifacts.secret_shares[i],
			&z_artifacts.secret_shares[i]).unwrap()).collect();

		// players can interpolate the polynomial of degree 2t and compute u && inv(u):
		let u_inv = compute_ecdsa_inversed_secret_coeff_from_shares(t,
			&artifacts.id_numbers.iter().take(2*t + 1).cloned().collect::<Vec<_>>(),
			&ui.iter().take(2*t + 1).cloned().collect::<Vec<_>>()).unwrap();

		// each player Pi computes his share of inv(x) as e[i] * inv(u)
		let x_inv_shares: Vec<_> = (0..n).map(|i| {
			let mut x_inv_share = e_artifacts.secret_shares[i].clone();
			x_inv_share.mul(&u_inv).unwrap();
			x_inv_share
		}).collect();

		x_inv_shares
	}

	pub fn do_encryption_and_decryption(t: usize, joint_public: &Public, id_numbers: &[Secret], secret_shares: &[Secret], joint_secret: Option<&Secret>, document_secret_plain: Public) -> (Public, Public) {
		// === PART2: encryption using joint public key ===

		// the next line is executed on KeyServer-client
		let encrypted_secret = encrypt_secret(&document_secret_plain, &joint_public).unwrap();

		// === PART3: decryption ===

		// next line is executed on KeyServer client
		let access_key = generate_random_scalar().unwrap();

		// use t + 1 nodes to compute joint shadow point
		let nodes_shadows: Vec<_> = (0..t + 1).map(|i|
			compute_node_shadow(&secret_shares[i], &id_numbers[i], id_numbers.iter()
				.enumerate()
				.filter(|&(j, _)| j != i)
				.take(t)
				.map(|(_, id_number)| id_number)).unwrap()).collect();

		let nodes_shadow_points: Vec<_> = nodes_shadows.iter()
			.map(|s| compute_node_shadow_point(&access_key, &encrypted_secret.common_point, s, None).unwrap())
			.map(|sp| sp.0)
			.collect();

		assert_eq!(nodes_shadows.len(), t + 1);
		assert_eq!(nodes_shadow_points.len(), t + 1);

		let joint_shadow_point = compute_joint_shadow_point(nodes_shadow_points.iter()).unwrap();
		let joint_shadow_point_test = compute_joint_shadow_point_test(&access_key, &encrypted_secret.common_point, nodes_shadows.iter()).unwrap();
		assert_eq!(joint_shadow_point, joint_shadow_point_test);

		// decrypt encrypted secret using joint shadow point
		let document_secret_decrypted = decrypt_with_joint_shadow(t, &access_key, &encrypted_secret.encrypted_point, &joint_shadow_point).unwrap();

		// decrypt encrypted secret using joint secret [just for test]
		let document_secret_decrypted_test = match joint_secret {
			Some(joint_secret) => decrypt_with_joint_secret(&encrypted_secret.encrypted_point, &encrypted_secret.common_point, joint_secret).unwrap(),
			None => document_secret_decrypted.clone(),
		};

		(document_secret_decrypted, document_secret_decrypted_test)
	}

	#[test]
	fn full_encryption_math_session() {
		let test_cases = [(0, 2), (1, 2), (1, 3), (2, 3), (1, 4), (2, 4), (3, 4), (1, 5), (2, 5), (3, 5), (4, 5),
			(1, 10), (2, 10), (3, 10), (4, 10), (5, 10), (6, 10), (7, 10), (8, 10), (9, 10)];
		for &(t, n) in &test_cases {
			let artifacts = run_key_generation(t, n, None, None);

			// compute joint private key [just for test]
			let joint_secret = compute_joint_secret(artifacts.polynoms1.iter().map(|p| &p[0])).unwrap();
			let joint_key_pair = KeyPair::from_secret(joint_secret.clone()).unwrap();
			assert_eq!(&artifacts.joint_public, joint_key_pair.public());

			// check secret shares computation [just for test]
			let secret_shares_polynom: Vec<_> = (0..t + 1).map(|k| compute_secret_share(artifacts.polynoms1.iter().map(|p| &p[k])).unwrap()).collect();
			let secret_shares_calculated_from_polynom: Vec<_> = artifacts.id_numbers.iter().map(|id_number| compute_polynom(&*secret_shares_polynom, id_number).unwrap()).collect();
			assert_eq!(artifacts.secret_shares, secret_shares_calculated_from_polynom);

			// now encrypt and decrypt data
			let document_secret_plain = generate_random_point().unwrap();
			let (document_secret_decrypted, document_secret_decrypted_test) =
				do_encryption_and_decryption(t, &artifacts.joint_public, &artifacts.id_numbers, &artifacts.secret_shares, Some(&joint_secret), document_secret_plain.clone());

			assert_eq!(document_secret_plain, document_secret_decrypted_test);
			assert_eq!(document_secret_plain, document_secret_decrypted);
		}
	}

	#[test]
	fn local_signature_works() {
		let key_pair = Random.generate().unwrap();
		let message_hash = "0000000000000000000000000000000000000000000000000000000000000042".parse().unwrap();
		let nonce = generate_random_scalar().unwrap();
		let signature = local_compute_schnorr_signature(&nonce, key_pair.secret(), &message_hash).unwrap();
		assert_eq!(verify_schnorr_signature(key_pair.public(), &signature, &message_hash), Ok(true));
	}

	#[test]
	fn full_schnorr_signature_math_session() {
		let test_cases = [(0, 1), (0, 2), (1, 2), (1, 3), (2, 3), (1, 4), (2, 4), (3, 4), (1, 5), (2, 5), (3, 5), (4, 5),
			(1, 10), (2, 10), (3, 10), (4, 10), (5, 10), (6, 10), (7, 10), (8, 10), (9, 10)];
		for &(t, n) in &test_cases {
			// hash of the message to be signed
			let message_hash: Secret = "0000000000000000000000000000000000000000000000000000000000000042".parse().unwrap();

			// === MiDS-S algorithm ===
			// setup: all nodes share master secret key && every node knows master public key
			let artifacts = run_key_generation(t, n, None, None);

			// in this gap (not related to math):
			// master node should ask every other node if it is able to do a signing
			// if there are < than t+1 nodes, able to sign => error
			// select t+1 nodes for signing session
			// all steps below are for this subset of nodes
			let n = t + 1;

			// step 1: run DKG to generate one-time secret key (nonce)
			let id_numbers = artifacts.id_numbers.iter().cloned().take(n).collect();
			let one_time_artifacts = run_key_generation(t, n, Some(id_numbers), None);

			// step 2: message hash && x coordinate of one-time public value are combined
			let combined_hash = combine_message_hash_with_public(&message_hash, &one_time_artifacts.joint_public).unwrap();

			// step 3: compute signature shares
			let partial_signatures: Vec<_> = (0..n)
				.map(|i| compute_schnorr_signature_share(
					t,
					&combined_hash,
					&one_time_artifacts.polynoms1[i][0],
					&artifacts.secret_shares[i],
					&artifacts.id_numbers[i],
					artifacts.id_numbers.iter()
						.enumerate()
						.filter(|&(j, _)| i != j)
						.map(|(_, n)| n)
						.take(t)
				).unwrap())
				.collect();

			// step 4: receive and verify signatures shares from other nodes
			let received_signatures: Vec<Vec<_>> = (0..n)
				.map(|i| (0..n)
					.filter(|j| i != *j)
					.map(|j| {
						let signature_share = partial_signatures[j].clone();
						assert!(_check_schnorr_signature_share(&combined_hash,
							&signature_share,
							&artifacts.public_shares[j],
							&one_time_artifacts.public_shares[j],
							artifacts.id_numbers.iter().take(t)).unwrap_or(false));
						signature_share
					})
					.collect())
				.collect();

			// step 5: compute signature
			let signatures: Vec<_> = (0..n)
				.map(|i| (combined_hash.clone(), compute_schnorr_signature(received_signatures[i].iter().chain(once(&partial_signatures[i]))).unwrap()))
				.collect();

			// === verify signature ===
			let master_secret = compute_joint_secret(artifacts.polynoms1.iter().map(|p| &p[0])).unwrap();
			let nonce = compute_joint_secret(one_time_artifacts.polynoms1.iter().map(|p| &p[0])).unwrap();
			let local_signature = local_compute_schnorr_signature(&nonce, &master_secret, &message_hash).unwrap();
			for signature in &signatures {
				assert_eq!(signature, &local_signature);
				assert_eq!(verify_schnorr_signature(&artifacts.joint_public, signature, &message_hash), Ok(true));
			}
		}
	}

	#[test]
	fn full_ecdsa_signature_math_session() {
		let test_cases = [(2, 5), (2, 6), (3, 11), (4, 11)];
		for &(t, n) in &test_cases {
			// values that can be hardcoded
			let joint_secret: Secret = Random.generate().unwrap().secret().clone();
			let joint_nonce: Secret = Random.generate().unwrap().secret().clone();
			let message_hash: H256 = H256::random();

			// convert message hash to EC scalar
			let message_hash_scalar = to_scalar(message_hash.clone()).unwrap();

			// generate secret key shares
			let artifacts = run_key_generation(t, n, None, Some(joint_secret));

			// generate nonce shares
			let nonce_artifacts = run_key_generation(t, n, Some(artifacts.id_numbers.clone()), Some(joint_nonce));

			// compute nonce public
			// x coordinate (mapped to EC field) of this public is the r-portion of signature
			let nonce_public_shares: Vec<_> = (0..n).map(|i| compute_public_share(&nonce_artifacts.polynoms1[i][0]).unwrap()).collect();
			let nonce_public = compute_joint_public(nonce_public_shares.iter()).unwrap();
			let signature_r = compute_ecdsa_r(&nonce_public).unwrap();

			// compute shares of inv(nonce) so that both nonce && inv(nonce) are still unknown to all nodes
			let nonce_inv_shares = run_reciprocal_protocol(t, &nonce_artifacts);

			// compute multiplication of secret-shares * inv-nonce-shares
			let mul_shares = run_multiplication_protocol(t, &artifacts.secret_shares, &nonce_inv_shares);

			// compute shares for s portion of signature: nonce_inv * (message_hash + secret * signature_r)
			// every node broadcasts this share
			let double_t = 2 * t;
			let signature_s_shares: Vec<_> = (0..double_t+1).map(|i| compute_ecdsa_s_share(
				&nonce_inv_shares[i],
				&mul_shares[i],
				&signature_r,
				&message_hash_scalar
			).unwrap()).collect();

			// compute signature_s from received shares
			let signature_s = compute_ecdsa_s(t,
				&signature_s_shares,
				&artifacts.id_numbers.iter().take(double_t + 1).cloned().collect::<Vec<_>>()
			).unwrap();

			// check signature
			let signature_actual = serialize_ecdsa_signature(&nonce_public, signature_r, signature_s);
			let joint_secret = compute_joint_secret(artifacts.polynoms1.iter().map(|p| &p[0])).unwrap();
			let joint_secret_pair = KeyPair::from_secret(joint_secret).unwrap();
			assert_eq!(recover(&signature_actual, &message_hash).unwrap(), *joint_secret_pair.public());
			assert!(verify_public(joint_secret_pair.public(), &signature_actual, &message_hash).unwrap());
		}
	}

	#[test]
	fn full_generation_math_session_with_refreshing_shares() {
		let test_cases = vec![(1, 4), (6, 10)];
		for (t, n) in test_cases {
			// generate key using t-of-n session
			let artifacts1 = run_key_generation(t, n, None, None);
			let joint_secret1 = compute_joint_secret(artifacts1.polynoms1.iter().map(|p1| &p1[0])).unwrap();

			// let's say we want to refresh existing secret shares
			// by doing this every T seconds, and assuming that in each T-second period adversary KS is not able to collect t+1 secret shares
			// we can be sure that the scheme is secure
			let artifacts2 = run_key_share_refreshing(t, t, n, &artifacts1);
			let joint_secret2 = compute_joint_secret_from_shares(t, &artifacts2.secret_shares.iter().take(t + 1).collect::<Vec<_>>(),
				&artifacts2.id_numbers.iter().take(t + 1).collect::<Vec<_>>()).unwrap();
			assert_eq!(joint_secret1, joint_secret2);

			// refresh again
			let artifacts3 = run_key_share_refreshing(t, t, n, &artifacts2);
			let joint_secret3 = compute_joint_secret_from_shares(t, &artifacts3.secret_shares.iter().take(t + 1).collect::<Vec<_>>(),
				&artifacts3.id_numbers.iter().take(t + 1).collect::<Vec<_>>()).unwrap();
			assert_eq!(joint_secret1, joint_secret3);
		}
	}

	#[test]
	fn full_generation_math_session_with_adding_new_nodes() {
		let test_cases = vec![(1, 3), (1, 4), (6, 10)];
		for (t, n) in test_cases {
			// generate key using t-of-n session
			let artifacts1 = run_key_generation(t, n, None, None);
			let joint_secret1 = compute_joint_secret(artifacts1.polynoms1.iter().map(|p1| &p1[0])).unwrap();

			// let's say we want to include additional couple of servers to the set
			// so that scheme becames t-of-n+2
			let artifacts2 = run_key_share_refreshing(t, t, n + 2, &artifacts1);
			let joint_secret2 = compute_joint_secret_from_shares(t, &artifacts2.secret_shares.iter().take(t + 1).collect::<Vec<_>>(),
				&artifacts2.id_numbers.iter().take(t + 1).collect::<Vec<_>>()).unwrap();
			assert_eq!(joint_secret1, joint_secret2);

			// include another server (t-of-n+3)
			let artifacts3 = run_key_share_refreshing(t, t, n + 3, &artifacts2);
			let joint_secret3 = compute_joint_secret_from_shares(t, &artifacts3.secret_shares.iter().take(t + 1).collect::<Vec<_>>(),
				&artifacts3.id_numbers.iter().take(t + 1).collect::<Vec<_>>()).unwrap();
			assert_eq!(joint_secret1, joint_secret3);
		}
	}

	#[test]
	fn full_generation_math_session_with_decreasing_threshold() {
		let (t, n) = (3, 5);

		// generate key using t-of-n session
		let artifacts1 = run_key_generation(t, n, None, None);

		let joint_secret1 = compute_joint_secret(artifacts1.polynoms1.iter().map(|p1| &p1[0])).unwrap();

		// let's say we want to decrease threshold so that it becames (t-1)-of-n
		let new_t = t - 1;
		let artifacts2 = run_key_share_refreshing(t, new_t, n, &artifacts1);
		let joint_secret2 = compute_joint_secret_from_shares(new_t, &artifacts2.secret_shares.iter().take(new_t + 1).collect::<Vec<_>>(),
			&artifacts2.id_numbers.iter().take(new_t + 1).collect::<Vec<_>>()).unwrap();
		assert_eq!(joint_secret1, joint_secret2);

		// let's say we want to decrease threshold once again so that it becames (t-2)-of-n
		let t = t - 1;
		let new_t = t - 2;
		let artifacts3 = run_key_share_refreshing(t, new_t, n, &artifacts2);
		let joint_secret3 = compute_joint_secret_from_shares(new_t, &artifacts3.secret_shares.iter().take(new_t + 1).collect::<Vec<_>>(),
			&artifacts3.id_numbers.iter().take(new_t + 1).collect::<Vec<_>>()).unwrap();
		assert_eq!(joint_secret1, joint_secret3);
	}

	#[test]
	fn full_zero_secret_generation_math_session() {
		let test_cases = vec![(1, 4), (2, 4)];
		for (t, n) in test_cases {
			// run joint zero generation session
			let id_numbers: Vec<_> = (0..n).map(|_| generate_random_scalar().unwrap()).collect();
			let artifacts = run_zero_key_generation(t, n, &id_numbers);

			// check that zero secret is generated
			// we can't compute secrets sum here, because result will be zero (invalid secret, unsupported by SECP256k1)
			// so just use complement trick: x + (-x) = 0
			// TODO [Refac]: switch to SECP256K1-free scalar EC arithmetic
			let partial_joint_secret = compute_secret_sum(artifacts.polynoms1.iter().take(n - 1).map(|p| &p[0])).unwrap();
			let mut partial_joint_secret_complement = artifacts.polynoms1[n - 1][0].clone();
			partial_joint_secret_complement.neg().unwrap();
			assert_eq!(partial_joint_secret, partial_joint_secret_complement);
		}
	}

	#[test]
	fn full_generation_with_multiplication() {
		let test_cases = vec![(1, 3), (2, 5), (2, 7), (3, 8)];
		for (t, n) in test_cases {
			// generate two shared secrets
			let artifacts1 = run_key_generation(t, n, None, None);
			let artifacts2 = run_key_generation(t, n, Some(artifacts1.id_numbers.clone()), None);

			// multiplicate original secrets
			let joint_secret1 = compute_joint_secret(artifacts1.polynoms1.iter().map(|p| &p[0])).unwrap();
			let joint_secret2 = compute_joint_secret(artifacts2.polynoms1.iter().map(|p| &p[0])).unwrap();
			let mut expected_joint_secret_mul = joint_secret1;
			expected_joint_secret_mul.mul(&joint_secret2).unwrap();

			// run multiplication protocol
			let joint_secret_mul_shares = run_multiplication_protocol(t, &artifacts1.secret_shares, &artifacts2.secret_shares);

			// calculate actual secrets multiplication
			let double_t = t * 2;
			let actual_joint_secret_mul = compute_joint_secret_from_shares(double_t,
				&joint_secret_mul_shares.iter().take(double_t + 1).collect::<Vec<_>>(),
				&artifacts1.id_numbers.iter().take(double_t + 1).collect::<Vec<_>>()).unwrap();

			assert_eq!(actual_joint_secret_mul, expected_joint_secret_mul);
		}
	}

	#[test]
	fn full_generation_with_reciprocal() {
		let test_cases = vec![(1, 3), (2, 5), (2, 7), (2, 7), (3, 8)];
		for (t, n) in test_cases {
			// generate shared secret
			let artifacts = run_key_generation(t, n, None, None);

			// calculate inversion of original shared secret
			let joint_secret = compute_joint_secret(artifacts.polynoms1.iter().map(|p| &p[0])).unwrap();
			let mut expected_joint_secret_inv = joint_secret.clone();
			expected_joint_secret_inv.inv().unwrap();

			// run inversion protocol
			let reciprocal_shares = run_reciprocal_protocol(t, &artifacts);

			// calculate actual secret inversion
			let double_t = t * 2;
			let actual_joint_secret_inv = compute_joint_secret_from_shares(double_t,
				&reciprocal_shares.iter().take(double_t + 1).collect::<Vec<_>>(),
				&artifacts.id_numbers.iter().take(double_t + 1).collect::<Vec<_>>()).unwrap();

			assert_eq!(actual_joint_secret_inv, expected_joint_secret_inv);
		}
	}
}
