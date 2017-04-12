// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use ethkey::{Public, Secret, Random, Generator, math};
use key_server_cluster::Error;

#[derive(Debug)]
/// Encryption result.
pub struct EncryptedSecret {
	/// Common encryption point.
	pub common_point: Public,
	/// Ecnrypted point.
	pub encrypted_point: Public,
}

/// Generate random scalar
pub fn generate_random_scalar() -> Result<Secret, Error> {
	Ok(Random.generate()?.secret().clone())
}

/// Generate random point
pub fn generate_random_point() -> Result<Public, Error> {
	Ok(Random.generate()?.public().clone())
}

/// Update point by multiplying to random scalar
pub fn update_random_point(point: &mut Public) -> Result<(), Error> {
	Ok(math::public_mul_secret(point, &generate_random_scalar()?)?)
}

/// Generate random polynom of threshold degree
pub fn generate_random_polynom(threshold: usize) -> Result<Vec<Secret>, Error> {
	let mut polynom: Vec<_> = Vec::with_capacity(threshold + 1);
	for _ in 0..threshold + 1 {
		polynom.push(generate_random_scalar()?);
	}
	debug_assert_eq!(polynom.len(), threshold + 1);
	Ok(polynom)
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

/// Compute secret share.
pub fn compute_secret_share<'a, I>(mut secret_values: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	let mut secret_share = secret_values.next().expect("compute_secret_share is called when cluster has at least one node; qed").clone();
	while let Some(secret_value) = secret_values.next() {
		secret_share.add(secret_value)?;
	}
	Ok(secret_share)
}

/// Compute public key share.
pub fn compute_public_share(self_secret_value: &Secret) -> Result<Public, Error> {
	let mut public_share = math::generation_point();
	math::public_mul_secret(&mut public_share, self_secret_value)?;
	Ok(public_share)
}

/// Compute joint public key.
pub fn compute_joint_public<'a, I>(mut public_shares: I) -> Result<Public, Error> where I: Iterator<Item=&'a Public> {
	let mut joint_public = public_shares.next().expect("compute_joint_public is called when cluster has at least one node; qed").clone();
	while let Some(public_share) = public_shares.next() {
		math::public_add(&mut joint_public, &public_share)?;
	}
	Ok(joint_public)
}

#[cfg(test)]
/// Compute joint secret key.
pub fn compute_joint_secret<'a, I>(mut secret_coeffs: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	let mut joint_secret = secret_coeffs.next().expect("compute_joint_private is called when cluster has at least one node; qed").clone();
	while let Some(secret_coeff) = secret_coeffs.next() {
		joint_secret.add(secret_coeff)?;
	}
	Ok(joint_secret)
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
pub fn compute_node_shadow<'a, I>(node_number: &Secret, node_secret_share: &Secret, mut other_nodes_numbers: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	let other_node_number = match other_nodes_numbers.next() {
		Some(other_node_number) => other_node_number,
		None => return Ok(node_secret_share.clone()),
	};

	let mut shadow = node_number.clone();
	shadow.sub(other_node_number)?;
	shadow.inv()?;
	shadow.mul(other_node_number)?;
	while let Some(other_node_number) = other_nodes_numbers.next() {
		let mut shadow_element = node_number.clone();
		shadow_element.sub(other_node_number)?;
		shadow_element.inv()?;
		shadow_element.mul(other_node_number)?;
		shadow.mul(&shadow_element)?;
	}

	shadow.mul(&node_secret_share)?;
	Ok(shadow)
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
pub fn compute_joint_shadow_point<'a, I>(mut nodes_shadow_points: I) -> Result<Public, Error> where I: Iterator<Item=&'a Public> {
	let mut joint_shadow_point = nodes_shadow_points.next().expect("compute_joint_shadow_point is called when at least two nodes are required to decrypt secret; qed").clone();
	while let Some(node_shadow_point) = nodes_shadow_points.next() {
		math::public_add(&mut joint_shadow_point, &node_shadow_point)?;
	}
	Ok(joint_shadow_point)
}

#[cfg(test)]
/// Compute joint shadow point (version for tests).
pub fn compute_joint_shadow_point_test<'a, I>(access_key: &Secret, common_point: &Public, mut nodes_shadows: I) -> Result<Public, Error> where I: Iterator<Item=&'a Secret> {
	let mut joint_shadow = nodes_shadows.next().expect("compute_joint_shadow_point_test is called when at least two nodes are required to decrypt secret; qed").clone();
	while let Some(node_shadow) = nodes_shadows.next() {
		joint_shadow.add(node_shadow)?;
	}
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

#[cfg(test)]
/// Decrypt shadow-encrypted secret.
pub fn decrypt_with_shadow_coefficients(mut decrypted_shadow: Public, mut common_shadow_point: Public, shadow_coefficients: Vec<Secret>) -> Result<Public, Error> {
	let mut shadow_coefficients_sum = shadow_coefficients[0].clone();
	for shadow_coefficient in shadow_coefficients.iter().skip(1) {
		shadow_coefficients_sum.add(shadow_coefficient)?;
	}
	math::public_mul_secret(&mut common_shadow_point, &shadow_coefficients_sum)?;
	math::public_add(&mut decrypted_shadow, &common_shadow_point)?;
	Ok(decrypted_shadow)
}

#[cfg(test)]
/// Decrypt data using joint secret (version for tests).
pub fn decrypt_with_joint_secret(encrypted_point: &Public, common_point: &Public, joint_secret: &Secret) -> Result<Public, Error> {
	let mut common_point_mul = common_point.clone();
	math::public_mul_secret(&mut common_point_mul, joint_secret)?;

	let mut decrypted_point = encrypted_point.clone();
	math::public_sub(&mut decrypted_point, &common_point_mul)?;

	Ok(decrypted_point)
}

#[cfg(test)]
pub mod tests {
	use ethkey::KeyPair;
	use super::*;

	pub fn do_encryption_and_decryption(t: usize, joint_public: &Public, id_numbers: &[Secret], secret_shares: &[Secret], joint_secret: Option<&Secret>, document_secret_plain: Public) -> (Public, Public) {
		// === PART2: encryption using joint public key ===

		// the next line is executed on KeyServer-client
		let encrypted_secret = encrypt_secret(&document_secret_plain, &joint_public).unwrap();

		// === PART3: decryption ===

		// next line is executed on KeyServer client
		let access_key = generate_random_scalar().unwrap();

		// use t + 1 nodes to compute joint shadow point
		let nodes_shadows: Vec<_> = (0..t + 1).map(|i|
			compute_node_shadow(&id_numbers[i], &secret_shares[i], id_numbers.iter()
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
			// === PART1: DKG ===
			
			// data, gathered during initialization
			let id_numbers: Vec<_> = (0..n).map(|_| generate_random_scalar().unwrap()).collect();

			// data, generated during keys dissemination
			let polynoms1: Vec<_> = (0..n).map(|_| generate_random_polynom(t).unwrap()).collect();
			let secrets1: Vec<_> = (0..n).map(|i| (0..n).map(|j| compute_polynom(&polynoms1[i], &id_numbers[j]).unwrap()).collect::<Vec<_>>()).collect();

			// data, generated during keys generation
			let public_shares: Vec<_> = (0..n).map(|i| compute_public_share(&polynoms1[i][0]).unwrap()).collect();
			let secret_shares: Vec<_> = (0..n).map(|i| compute_secret_share(secrets1.iter().map(|s| &s[i])).unwrap()).collect();

			// joint public key, as a result of DKG
			let joint_public = compute_joint_public(public_shares.iter()).unwrap();

			// compute joint private key [just for test]
			let joint_secret = compute_joint_secret(polynoms1.iter().map(|p| &p[0])).unwrap();
			let joint_key_pair = KeyPair::from_secret(joint_secret.clone()).unwrap();
			assert_eq!(&joint_public, joint_key_pair.public());

			// check secret shares computation [just for test]
			let secret_shares_polynom: Vec<_> = (0..t + 1).map(|k| compute_secret_share(polynoms1.iter().map(|p| &p[k])).unwrap()).collect();
			let secret_shares_calculated_from_polynom: Vec<_> = id_numbers.iter().map(|id_number| compute_polynom(&*secret_shares_polynom, id_number).unwrap()).collect();
			assert_eq!(secret_shares, secret_shares_calculated_from_polynom);

			// now encrypt and decrypt data
			let document_secret_plain = generate_random_point().unwrap();
			let (document_secret_decrypted, document_secret_decrypted_test) =
				do_encryption_and_decryption(t, &joint_public, &id_numbers, &secret_shares, Some(&joint_secret), document_secret_plain.clone());

			assert_eq!(document_secret_plain, document_secret_decrypted_test);
			assert_eq!(document_secret_plain, document_secret_decrypted);
		}
	}
}
