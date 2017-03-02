use std::collections::BTreeMap;
use ethkey::{Public, Secret, math};
use key_server_cluster::{Error, NodeId};

#[derive(Debug, Clone)]
/// Data, collected during encryption session.
pub struct DecryptionData {
	/// Public knowledge of nodes, participated in encryption session.
	pub nodes: BTreeMap<NodeId, NodeData>,
}

#[derive(Debug, Clone)]
pub struct NodeData {
	/// Secret node identification number.
	pub id_number: Secret,
	/// Secret key share.
	pub secret_share: Secret,
}

pub fn do_decryption_dummy(access_key: &Secret, common_point: &Public, encrypted_point: &Public, decryption_data: &DecryptionData) -> Result<Public, Error> {
	// calculate shadow (this is subject to change as this must be calculated privately on every cluster node)
	let mut node_shadows = Vec::new();
	let mut shadow_points = Vec::new();
	for (node_id, node_data) in decryption_data.nodes.clone() {
		let other_nodes: Vec<_> = decryption_data.nodes.iter()
			.filter(|&(other_node_id, _)| &node_id != other_node_id)
			.map(|(_, other_node_data)| other_node_data.id_number.clone())
			.collect();

		let node_shadow = calculate_shadow(&node_data.id_number, &node_data.secret_share, &other_nodes)?;
		let node_shadow_point = calculate_shadow_point(&access_key, &common_point, &node_shadow)?;
		node_shadows.push(node_shadow);
		shadow_points.push(node_shadow_point);
	}

	// all calculations below are done on 'master' KS
	let joint_shadow_point1 = calculate_joint_shadow_point(&shadow_points)?;
	let joint_shadow_point2 = calculate_joint_shadow_point2(access_key, &node_shadows, common_point)?;
	assert_eq!(joint_shadow_point1, joint_shadow_point2); // just to check that we have computed it correctly
	decrypt(&access_key, &encrypted_point, &joint_shadow_point1)
}

fn calculate_shadow(id_number: &Secret, secret_share: &Secret, nodes_id_numbers: &[Secret]) -> Result<Secret, Error> {
	let mut iter = nodes_id_numbers.iter();
	let node1_id_number = iter.next().expect("at least two nodes must participate in cluster decryption; qed");
	let mut shadow = id_number.clone();
	math::secret_sub(&mut shadow, &node1_id_number)?;
	math::secret_inv(&mut shadow)?;
	math::secret_mul(&mut shadow, &node1_id_number)?;
	while let Some(node_id_number) = iter.next() {
		let mut shadow_element = id_number.clone();
		math::secret_sub(&mut shadow_element, &node_id_number)?;
		math::secret_inv(&mut shadow_element)?;
		math::secret_mul(&mut shadow_element, &node_id_number)?;
		math::secret_mul(&mut shadow, &shadow_element)?;
	}

	math::secret_mul(&mut shadow, &secret_share)?;
	Ok(shadow)
}

fn calculate_shadow_point(access_key: &Secret, common_point: &Public, shadow: &Secret) -> Result<Public, Error> {
	let mut shadow_key = access_key.clone();
	math::secret_mul(&mut shadow_key, shadow)?;
	let mut shadow_point = common_point.clone();
	math::public_mul_secret(&mut shadow_point, &shadow_key)?;
	Ok(shadow_point)
}

fn calculate_joint_shadow_point(shadow_points: &[Public]) -> Result<Public, Error> {
	let mut joint_shadow_point = shadow_points[0].clone();
	for shadow_point in shadow_points.iter().skip(1) {
		math::public_add(&mut joint_shadow_point, &shadow_point)?;
	}
	Ok(joint_shadow_point)
}

fn calculate_joint_shadow_point2(access_key: &Secret, nodes_shadows: &[Secret], common_point: &Public) -> Result<Public, Error> {
	let mut nodes_shadows_iter = nodes_shadows.iter();
	let mut common_node_shadow = nodes_shadows_iter.next().unwrap().clone();
	while let Some(node_shadow) = nodes_shadows_iter.next() {
		math::secret_add(&mut common_node_shadow, node_shadow).unwrap();
	}
	math::secret_mul(&mut common_node_shadow, access_key).unwrap();
	let mut joint_shadow_point = common_point.clone();
	math::public_mul_secret(&mut joint_shadow_point, &common_node_shadow).unwrap();
	Ok(joint_shadow_point)
}

fn decrypt(access_key: &Secret, encrypted_point: &Public, joint_shadow_point: &Public) -> Result<Public, Error> {
	let mut inv_access_key = access_key.clone();
	math::secret_inv(&mut inv_access_key)?;
	
	let mut decrypted_point = joint_shadow_point.clone();
	math::public_mul_secret(&mut decrypted_point, &inv_access_key)?;
	math::public_add(&mut decrypted_point, encrypted_point)?;

	Ok(decrypted_point)
}
