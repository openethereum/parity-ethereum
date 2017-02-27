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
