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
use bigint::prelude::U256;
use bigint::hash::H256;
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

/// Generate random scalar
pub fn generate_random_scalar() -> Result<Secret, Error> {
	Ok(Random.generate()?.secret().clone())
}

/// Generate random point
pub fn generate_random_point() -> Result<Public, Error> {
	Ok(Random.generate()?.public().clone())
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

/// Compute absolute term of additional polynom1 when new node is added to the existing generation node set
#[cfg(test)]
pub fn compute_additional_polynom1_absolute_term<'a, I>(secret_values: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	let mut absolute_term = compute_secret_sum(secret_values)?;
	absolute_term.neg()?;
	Ok(absolute_term)
}

/// Add two polynoms together (coeff = coeff1 + coeff2).
#[cfg(test)]
pub fn add_polynoms(polynom1: &[Secret], polynom2: &[Secret], is_absolute_term2_zero: bool) -> Result<Vec<Secret>, Error> {
	polynom1.iter().zip(polynom2.iter())
		.enumerate()
		.map(|(i, (c1, c2))| {
			let mut sum_coeff = c1.clone();
			if !is_absolute_term2_zero || i != 0 {
				sum_coeff.add(c2)?;
			}
			Ok(sum_coeff)
		})
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

/// Check refreshed keys passed by other participants.
#[cfg(test)]
pub fn refreshed_keys_verification(threshold: usize, number_id: &Secret, secret1: &Secret, publics: &[Public]) -> Result<bool, Error> {
	// calculate left part
	let mut left = math::generation_point();
	math::public_mul_secret(&mut left, secret1)?;

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

/// Compute joint secret key.
#[cfg(test)]
pub fn compute_joint_secret<'a, I>(secret_coeffs: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	compute_secret_sum(secret_coeffs)
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
	let hash: U256 = hash.into();
	let hash: H256 = (hash % math::curve_order()).into();
	let hash = Secret::from_slice(&*hash);
	hash.check_validity()?;

	Ok(hash)
}

/// Compute signature share.
pub fn compute_signature_share<'a, I>(threshold: usize, combined_hash: &Secret, one_time_secret_coeff: &Secret, node_secret_share: &Secret, node_number: &Secret, other_nodes_numbers: I)
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

/// Check signature share.
pub fn _check_signature_share<'a, I>(_combined_hash: &Secret, _signature_share: &Secret, _public_share: &Public, _one_time_public_share: &Public, _node_numbers: I)
	-> Result<bool, Error> where I: Iterator<Item=&'a Secret> {
	// TODO: in paper partial signature is checked using comparison:
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

/// Compute signature.
pub fn compute_signature<'a, I>(signature_shares: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	compute_secret_sum(signature_shares)
}

/// Locally compute Schnorr signature as described in https://en.wikipedia.org/wiki/Schnorr_signature#Signing.
#[cfg(test)]
pub fn local_compute_signature(nonce: &Secret, secret: &Secret, message_hash: &Secret) -> Result<(Secret, Secret), Error> {
	let mut nonce_public = math::generation_point();
	math::public_mul_secret(&mut nonce_public, &nonce).unwrap();

	let combined_hash = combine_message_hash_with_public(message_hash, &nonce_public)?;

	let mut sig_subtrahend = combined_hash.clone();
	sig_subtrahend.mul(secret)?;
	let mut sig = nonce.clone();
	sig.sub(&sig_subtrahend)?;

	Ok((combined_hash, sig))
}

/// Verify signature as described in https://en.wikipedia.org/wiki/Schnorr_signature#Verifying.
#[cfg(test)]
pub fn verify_signature(public: &Public, signature: &(Secret, Secret), message_hash: &H256) -> Result<bool, Error> {
	let mut addendum = math::generation_point();
	math::public_mul_secret(&mut addendum, &signature.1)?;
	let mut nonce_public = public.clone();
	math::public_mul_secret(&mut nonce_public, &signature.0)?;
	math::public_add(&mut nonce_public, &addendum)?;

	let combined_hash = combine_message_hash_with_public(message_hash, &nonce_public)?;
	Ok(combined_hash == signature.0)
}

#[cfg(test)]
pub mod tests {
	use std::iter::once;
	use ethkey::KeyPair;
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

	fn run_key_generation(t: usize, n: usize, id_numbers: Option<Vec<Secret>>) -> KeyGenerationArtifacts {
		// === PART1: DKG ===

		// data, gathered during initialization
		let derived_point = Random.generate().unwrap().public().clone();
		let id_numbers: Vec<_> = match id_numbers {
			Some(id_numbers) => id_numbers,
			None => (0..n).map(|_| generate_random_scalar().unwrap()).collect(),
		};

		// data, generated during keys dissemination
		let polynoms1: Vec<_> = (0..n).map(|_| generate_random_polynom(t).unwrap()).collect();
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

	fn run_key_share_refreshing(t: usize, n: usize, artifacts: &KeyGenerationArtifacts) -> KeyGenerationArtifacts {
		// === share refreshing protocol from http://www.wu.ece.ufl.edu/mypapers/msig.pdf

		// key refreshing distribution algorithm (KRD)
		let refreshed_polynoms1: Vec<_> = (0..n).map(|_| generate_random_polynom(t).unwrap()).collect();
		let refreshed_polynoms1_sum: Vec<_> = (0..n).map(|i| add_polynoms(&artifacts.polynoms1[i], &refreshed_polynoms1[i], true).unwrap()).collect();
		let refreshed_secrets1: Vec<_> = (0..n).map(|i| (0..n).map(|j| compute_polynom(&refreshed_polynoms1_sum[i], &artifacts.id_numbers[j]).unwrap()).collect::<Vec<_>>()).collect();
		let refreshed_publics: Vec<_> = (0..n).map(|i| {
			(0..t+1).map(|j| compute_public_share(&refreshed_polynoms1_sum[i][j]).unwrap()).collect::<Vec<_>>()
		}).collect();

		// key refreshing verification algorithm (KRV)
		(0..n).map(|i| (0..n).map(|j| if i != j {
			assert!(refreshed_keys_verification(t, &artifacts.id_numbers[i], &refreshed_secrets1[j][i], &refreshed_publics[j]).unwrap())
		}).collect::<Vec<_>>()).collect::<Vec<_>>();

		// data, generated during keys generation
		let public_shares: Vec<_> = (0..n).map(|i| compute_public_share(&refreshed_polynoms1_sum[i][0]).unwrap()).collect();
		let secret_shares: Vec<_> = (0..n).map(|i| compute_secret_share(refreshed_secrets1.iter().map(|s| &s[i])).unwrap()).collect();

		// joint public key, as a result of DKG
		let joint_public = compute_joint_public(public_shares.iter()).unwrap();

		KeyGenerationArtifacts {
			id_numbers: artifacts.id_numbers.clone(),
			polynoms1: refreshed_polynoms1_sum,
			secrets1: refreshed_secrets1,
			public_shares: public_shares,
			secret_shares: secret_shares,
			joint_public: joint_public,
		}
	}

	fn run_key_share_refreshing_and_add_new_nodes(t: usize, n: usize, new_nodes: usize, artifacts: &KeyGenerationArtifacts) -> KeyGenerationArtifacts {
		// === share refreshing protocol (with new node addition) from http://www.wu.ece.ufl.edu/mypapers/msig.pdf
		let mut id_numbers: Vec<_> = artifacts.id_numbers.iter().cloned().collect();

		// key refreshing distribution algorithm (KRD)
		// for each new node: generate random polynom
		let refreshed_polynoms1: Vec<_> = (0..n).map(|_| (0..new_nodes).map(|_| generate_random_polynom(t).unwrap()).collect::<Vec<_>>()).collect();
		let mut refreshed_polynoms1_sum: Vec<_> = (0..n).map(|i| {
			let mut refreshed_polynom1_sum = artifacts.polynoms1[i].clone();
			for refreshed_polynom1 in &refreshed_polynoms1[i] {
				refreshed_polynom1_sum = add_polynoms(&refreshed_polynom1_sum, refreshed_polynom1, false).unwrap();
			}
			refreshed_polynom1_sum
		}).collect();

		// new nodes receiving private information and generates its own polynom
		let mut new_nodes_polynom1 = Vec::with_capacity(new_nodes);
		for i in 0..new_nodes {
			let mut new_polynom1 = generate_random_polynom(t).unwrap();
			let new_polynom_absolute_term = compute_additional_polynom1_absolute_term(refreshed_polynoms1.iter().map(|polynom1| &polynom1[i][0])).unwrap();
			new_polynom1[0] = new_polynom_absolute_term;
			new_nodes_polynom1.push(new_polynom1);
		}


		// new nodes sends its own information to all other nodes
		let n = n + new_nodes;
		id_numbers.extend((0..new_nodes).map(|_| Random.generate().unwrap().secret().clone()));
		refreshed_polynoms1_sum.extend(new_nodes_polynom1);

		// the rest of protocol is the same as without new node
		let refreshed_secrets1: Vec<_> = (0..n).map(|i| (0..n).map(|j| compute_polynom(&refreshed_polynoms1_sum[i], &id_numbers[j]).unwrap()).collect::<Vec<_>>()).collect();
		let refreshed_publics: Vec<_> = (0..n).map(|i| {
			(0..t+1).map(|j| compute_public_share(&refreshed_polynoms1_sum[i][j]).unwrap()).collect::<Vec<_>>()
		}).collect();

		// key refreshing verification algorithm (KRV)
		(0..n).map(|i| (0..n).map(|j| if i != j {
			assert!(refreshed_keys_verification(t, &id_numbers[i], &refreshed_secrets1[j][i], &refreshed_publics[j]).unwrap())
		}).collect::<Vec<_>>()).collect::<Vec<_>>();

		// data, generated during keys generation
		let public_shares: Vec<_> = (0..n).map(|i| compute_public_share(&refreshed_polynoms1_sum[i][0]).unwrap()).collect();
		let secret_shares: Vec<_> = (0..n).map(|i| compute_secret_share(refreshed_secrets1.iter().map(|s| &s[i])).unwrap()).collect();

		// joint public key, as a result of DKG
		let joint_public = compute_joint_public(public_shares.iter()).unwrap();

		KeyGenerationArtifacts {
			id_numbers: id_numbers,
			polynoms1: refreshed_polynoms1_sum,
			secrets1: refreshed_secrets1,
			public_shares: public_shares,
			secret_shares: secret_shares,
			joint_public: joint_public,
		}
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
			let artifacts = run_key_generation(t, n, None);

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
		let signature = local_compute_signature(&nonce, key_pair.secret(), &message_hash).unwrap();
		assert_eq!(verify_signature(key_pair.public(), &signature, &message_hash), Ok(true));
	}

	#[test]
	fn full_signature_math_session() {
		let test_cases = [(0, 1), (0, 2), (1, 2), (1, 3), (2, 3), (1, 4), (2, 4), (3, 4), (1, 5), (2, 5), (3, 5), (4, 5),
			(1, 10), (2, 10), (3, 10), (4, 10), (5, 10), (6, 10), (7, 10), (8, 10), (9, 10)];
		for &(t, n) in &test_cases {
			// hash of the message to be signed
			let message_hash: Secret = "0000000000000000000000000000000000000000000000000000000000000042".parse().unwrap();

			// === MiDS-S algorithm ===
			// setup: all nodes share master secret key && every node knows master public key
			let artifacts = run_key_generation(t, n, None);

			// in this gap (not related to math):
			// master node should ask every other node if it is able to do a signing
			// if there are < than t+1 nodes, able to sign => error
			// select t+1 nodes for signing session
			// all steps below are for this subset of nodes
			let n = t + 1;

			// step 1: run DKG to generate one-time secret key (nonce)
			let id_numbers = artifacts.id_numbers.iter().cloned().take(n).collect();
			let one_time_artifacts = run_key_generation(t, n, Some(id_numbers));

			// step 2: message hash && x coordinate of one-time public value are combined
			let combined_hash = combine_message_hash_with_public(&message_hash, &one_time_artifacts.joint_public).unwrap();

			// step 3: compute signature shares
			let partial_signatures: Vec<_> = (0..n)
				.map(|i| compute_signature_share(
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
						assert!(_check_signature_share(&combined_hash,
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
				.map(|i| (combined_hash.clone(), compute_signature(received_signatures[i].iter().chain(once(&partial_signatures[i]))).unwrap()))
				.collect();

			// === verify signature ===
			let master_secret = compute_joint_secret(artifacts.polynoms1.iter().map(|p| &p[0])).unwrap();
			let nonce = compute_joint_secret(one_time_artifacts.polynoms1.iter().map(|p| &p[0])).unwrap();
			let local_signature = local_compute_signature(&nonce, &master_secret, &message_hash).unwrap();
			for signature in &signatures {
				assert_eq!(signature, &local_signature);
				assert_eq!(verify_signature(&artifacts.joint_public, signature, &message_hash), Ok(true));
			}
		}
	}

	#[test]
	fn full_generation_math_session_with_refreshing_shares() {
		// generate key using 6-of-10 session
		let (t, n) = (5, 10);
		let artifacts1 = run_key_generation(t, n, None);

		// let's say we want to refresh existing secret shares
		// by doing this every T seconds, and assuming that in each T-second period adversary KS is not able to collect t+1 secret shares
		// we can be sure that the scheme is secure
		let artifacts2 = run_key_share_refreshing(t, n, &artifacts1);
		assert_eq!(artifacts1.joint_public, artifacts2.joint_public);

		// refresh again
		let artifacts3 = run_key_share_refreshing(t, n, &artifacts2);
		assert_eq!(artifacts1.joint_public, artifacts3.joint_public);
	}

	#[test]
	fn full_generation_math_session_with_adding_new_nodes() {
		// generate key using 6-of-10 session
		let (t, n) = (5, 10);
		let artifacts1 = run_key_generation(t, n, None);

		// let's say we want to include additional server to the set
		// so that scheme becames 6-of-11
		let artifacts2 = run_key_share_refreshing_and_add_new_nodes(t, n, 1, &artifacts1);
		assert_eq!(artifacts1.joint_public, artifacts2.joint_public);

		// include another couple of servers (6-of-13)
		let artifacts3 = run_key_share_refreshing_and_add_new_nodes(t, n + 1, 2, &artifacts2);
		assert_eq!(artifacts1.joint_public, artifacts3.joint_public);
	}
}
