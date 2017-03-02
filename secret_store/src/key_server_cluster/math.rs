use ethkey::{Public, Secret, Random, Generator, math};
use key_server_cluster::Error;

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
		let generate_random_scalar = Random.generate()?.secret().clone();
		polynom.push(generate_random_scalar);
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
		math::secret_pow(&mut appendum, i).unwrap();

		// calculate coeff * pow(point, i)
		let coeff = &polynom[i];
		math::secret_mul(&mut appendum, coeff).unwrap();

		// calculate result + coeff * pow(point, i)
		math::secret_add(&mut result, &appendum).unwrap();
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
		math::secret_pow(&mut secret_pow, i)?;

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
		math::secret_add(&mut secret_share, &secret_value)?;
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
		math::secret_add(&mut joint_secret, &secret_coeff)?;
	}
	Ok(joint_secret)
}

/// Encrypt secret with joint public key.
pub fn encrypt_secret(secret: Public, joint_public: &Public) -> Result<EncryptedSecret, Error> {
	// this is performed by KS-cluster client (or KS master)
	let key_pair = Random.generate()?;

	// k * T
	let mut common_point = math::generation_point();
	math::public_mul_secret(&mut common_point, key_pair.secret())?;

	// M + k * y
	let mut encrypted_point = joint_public.clone();
	math::public_mul_secret(&mut encrypted_point, key_pair.secret())?;
	math::public_add(&mut encrypted_point, &secret)?;

	Ok(EncryptedSecret {
		common_point: common_point,
		encrypted_point: encrypted_point,
	})
}

/// Compute shadow for the node.
pub fn compute_node_shadow<'a, I>(node_number: &Secret, node_secret_share: &Secret, mut other_nodes_numbers: I) -> Result<Secret, Error> where I: Iterator<Item=&'a Secret> {
	let other_node_number = other_nodes_numbers.next().expect("compute_node_shadow is called when at least two nodes are required to decrypt secret; qed");
	let mut shadow = node_number.clone();
	math::secret_sub(&mut shadow, other_node_number).unwrap();
	math::secret_inv(&mut shadow).unwrap();
	math::secret_mul(&mut shadow, other_node_number).unwrap();
	while let Some(other_node_number) = other_nodes_numbers.next() {
		let mut shadow_element = node_number.clone();
		math::secret_sub(&mut shadow_element, other_node_number).unwrap();
		math::secret_inv(&mut shadow_element).unwrap();
		math::secret_mul(&mut shadow_element, other_node_number).unwrap();
		math::secret_mul(&mut shadow, &shadow_element).unwrap();
	}

	math::secret_mul(&mut shadow, &node_secret_share).unwrap();
	Ok(shadow)
}

/// Compute shadow point for the node.
pub fn compute_node_shadow_point(access_key: &Secret, common_point: &Public, node_shadow: &Secret) -> Result<Public, Error> {
	let mut shadow_key = access_key.clone();
	math::secret_mul(&mut shadow_key, node_shadow)?;
	let mut node_shadow_point = common_point.clone();
	math::public_mul_secret(&mut node_shadow_point, &shadow_key)?;
	Ok(node_shadow_point)
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
		math::secret_add(&mut joint_shadow, node_shadow)?;
	}
	math::secret_mul(&mut joint_shadow, access_key)?;

	let mut joint_shadow_point = common_point.clone();
	math::public_mul_secret(&mut joint_shadow_point, &joint_shadow)?;
	Ok(joint_shadow_point)
}

/// Decrypt data using joint shadow point.
pub fn decrypt_with_joint_shadow(access_key: &Secret, encrypted_point: &Public, joint_shadow_point: &Public) -> Result<Public, Error> {
	let mut inv_access_key = access_key.clone();
	math::secret_inv(&mut inv_access_key)?;
	
	let mut decrypted_point = joint_shadow_point.clone();
	math::public_mul_secret(&mut decrypted_point, &inv_access_key)?;
	math::public_add(&mut decrypted_point, encrypted_point)?;

	Ok(decrypted_point)
}

#[cfg(test)]
pub mod tests {
	use ethkey::KeyPair;
	use super::*;

	pub fn do_encryption_and_decryption(t: usize, joint_public: &Public, id_numbers: &[Secret], secret_shares: &[Secret], document_secret_plain: Public) -> Public {
		// === PART2: encryption using joint public key ===

		// the next line is executed on KeyServer-client
		// so that secret is never seen by any KeyServer
		let encrypted_secret = encrypt_secret(document_secret_plain.clone(), &joint_public).unwrap();

		// === PART3: decryption ===

		// next line is executed on KeyServer client
		// so that secret is never seen by any KeyServer
		let access_key = generate_random_scalar().unwrap();

		// use t + 1 nodes to compute joint shadow point
		let nodes_shadows: Vec<_> = (0..t + 1).map(|i|
			compute_node_shadow(&id_numbers[i], &secret_shares[i], id_numbers.iter()
				.enumerate()
				.filter(|&(j, _)| j != i)
				.take(t)
				.map(|(_, id_number)| id_number)).unwrap()).collect();
		let nodes_shadow_points: Vec<_> = nodes_shadows.iter().map(|s| compute_node_shadow_point(&access_key, &encrypted_secret.common_point, s).unwrap()).collect();
		let joint_shadow_point = compute_joint_shadow_point(nodes_shadow_points.iter()).unwrap();
		let joint_shadow_point_test = compute_joint_shadow_point_test(&access_key, &encrypted_secret.common_point, nodes_shadows.iter()).unwrap();
		assert_eq!(joint_shadow_point, joint_shadow_point_test);

		// decrypt encrypted secret using joint shadow point
		// this is executed on KeyServer client
		// so that secret is never seen by any KeyServer
		let document_secret_decrypted = decrypt_with_joint_shadow(&access_key, &encrypted_secret.encrypted_point, &joint_shadow_point).unwrap();

		document_secret_decrypted
	}

	#[test]
	fn full_encryption_math_session() {
		let test_cases = [(3, 5)];
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

			// now encrypt and decrypt data
			let document_secret_plain = generate_random_point().unwrap();
			let document_secret_decrypted = do_encryption_and_decryption(t, &joint_public, &id_numbers, &secret_shares, document_secret_plain.clone());
			assert_eq!(document_secret_plain, document_secret_decrypted);
		}
	}
}
