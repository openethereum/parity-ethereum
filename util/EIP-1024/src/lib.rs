extern crate saltbabe;
#[macro_use]
extern crate arrayref;
extern crate hex;
use std::str;
pub use saltbabe::{KeyPair,Public, Secret, Error};
pub use saltbabe::traits::FromUnsafeSlice;

const VERSION: &str =  "x25519-xsalsa20-poly1305";

#[derive(Debug, Clone)]
pub struct EncryptedData {
    version: String,    
    nonce: String,
    ephem_public: String,
    ciphertext: String
}

pub fn get_encryption_keypair(sk: [u8; 32]) -> KeyPair<Secret, Public> {
    let keypair = saltbabe::crypto_box::gen_keypair_from_secret(&sk);
    return keypair;
}

pub fn to_byte32(bytes: &[u8]) -> [u8; 32] {
    array_ref!(bytes, 0, 32).clone()
}


pub fn encrypt(data: &[u8], version: Option<String>, send_pk: [u8; 32], ephermal_sk: [u8; 32]) -> Result<EncryptedData, Error> {
    let version = match version {
        Some(s) => s,
        None => VERSION.to_string()
    };
    let nonce = saltbabe::gen_nonce();
    let send_public = Public::from_unsafe_slice(&send_pk).unwrap();
    // Generate another ephemeral keypair from the key input
    let ephemeral_keypair = saltbabe::crypto_box::gen_keypair_from_secret(&ephermal_sk);
    let result = saltbabe::crypto_box::seal(&data, &nonce, &send_public, ephemeral_keypair.clone().secret()).unwrap();
    let result_hex = hex::encode(&result);
    let blob = EncryptedData {
        version: version,
        nonce: hex::encode(nonce),
        ephem_public: hex::encode(**ephemeral_keypair.public()),
        ciphertext: result_hex
    };
    return Ok(blob);
}


pub fn decrypt(encrypted_data: EncryptedData, recv_sk: [u8; 32]) -> Result<String, Error> {
    
    let recv_sk = KeyPair::<Secret, Public>::from_secret_slice(&recv_sk).unwrap();
    let pk_byte32 = to_byte32(&hex::decode(&encrypted_data.ephem_public).unwrap()); 
    let send_pk = Public::from_unsafe_slice(&pk_byte32).unwrap();
    let cipher_bytes = hex::decode(encrypted_data.ciphertext).unwrap();
    let nonce_bytes = hex::decode(encrypted_data.nonce).unwrap();
    let nonce = array_ref!(nonce_bytes, 0, 24).clone();
    println!("cipher_bytes: {:?}\n nonce: {:?}\n send_pk: {:?}\n recv_sk: {:?}\n", cipher_bytes, nonce, hex::encode(*send_pk), hex::encode(recv_sk.secret()) );
    let result = saltbabe::crypto_box::open(&cipher_bytes, &nonce, &send_pk, &recv_sk.secret()).unwrap();
    return Ok(str::from_utf8(&result.clone()).unwrap().to_string());
}

pub fn gen_keypair() -> KeyPair<Secret, Public> {
    
    KeyPair::<Secret, Public>::generate_keypair().unwrap()
    
}



#[cfg(test)]
mod tests {
    extern crate saltbabe;
    extern crate hex;
 

    #[test]
    fn get_encryption_publickey_works() {
        let recv_sk = "mJxmrVq8pfeR80HMZBTkjV+RiND1lqPqLuCdDUiduis=";
        let recv_sk_slice: [u8; 32] = crate::to_byte32(recv_sk.as_bytes());
        let public = **crate::get_encryption_keypair(recv_sk_slice).public();
        let pk_hex = hex::encode(public.clone());
        println!("Generated public: {}", pk_hex.clone());
        assert_eq!("262c59a6bb83b58a8120911bf6ed4863157089fc4bf0b294a95206d93146ad14", pk_hex);
    }
    #[test]
    fn to_encrypt() {
        let bob_sk = "mJxmrVq8pfeR80HMZBTkjV+RiND1lqPqLuCdDUiduis=";
        let bob_sk_slice: [u8; 32] = crate::to_byte32(bob_sk.as_bytes());
        let alice_sk = "Rz2i6pXUKcpWt6/b+mYtPPH+PiwhyLswOjcP8ZM0dyI=";
        let alice_sk_slice: [u8; 32] = crate::to_byte32(alice_sk.as_bytes());
        let alice = saltbabe::crypto_box::gen_keypair_from_secret(&bob_sk_slice);
        let bob = saltbabe::crypto_box::gen_keypair_from_secret(&alice_sk_slice);
        
        // Alice requests Bob's public encryption key so bob sends his encryption public key
        let bob_encrypt_pubkey = **crate::get_encryption_keypair(*bob.secret()).public();

        // Alice generates a random ephemeralKeyPair 
        let alice_ephemeral_keypair = saltbabe::crypto_box::gen_keypair_from_secret(alice.secret());

        // Alice uses her ephemeralKeypair.secretKey and Bob's encryptionPublicKey to encrypt the data using nacl.box.
        let encrypted = crate::encrypt(b"Hello world", None, bob_encrypt_pubkey, *alice_ephemeral_keypair.secret());

        // Alice sends encrypted blob to Bob
        println!("{:?}", encrypted);
    }

    #[test]
    fn to_decrypt() {
        let bob_sk = "mJxmrVq8pfeR80HMZBTkjV+RiND1lqPqLuCdDUiduis=";
        let bob_sk_slice: [u8; 32] = crate::to_byte32(bob_sk.as_bytes());
        let alice_sk = "Rz2i6pXUKcpWt6/b+mYtPPH+PiwhyLswOjcP8ZM0dyI=";
        let alice_sk_slice: [u8; 32] = crate::to_byte32(alice_sk.as_bytes());
        let alice = saltbabe::crypto_box::gen_keypair_from_secret(&bob_sk_slice);
        let bob = saltbabe::crypto_box::gen_keypair_from_secret(&alice_sk_slice);
        // Alice requests Bob's public encryption key so bob sends his encryption public key
        let bob_encrypt_keypair = crate::get_encryption_keypair(*bob.secret());

        // Alice generates a random ephemeralKeyPair 
        let alice_ephemeral_keypair = saltbabe::crypto_box::gen_keypair_from_secret(alice.secret());

        
        // Encrypt data first
        let encrypted_data = crate::encrypt(b"Hello world", None, **bob_encrypt_keypair.public(), *alice_ephemeral_keypair.secret()).unwrap();
        

        // Bob generates his encryptionPrivateKey
        let bob_encrypt_secret = bob_encrypt_keypair.secret(); 


        // Bob passes his encryptionPrivateKey
        // along with the encrypted blob 
        // to nacl.box.open(ciphertext, nonce, ephemPublicKey, myEncryptionPrivatekey)
        let decrypted = crate::decrypt(encrypted_data, *bob_encrypt_secret).unwrap();
        
        // Decrypted message
        println!("{:?}", decrypted);
    }
}
