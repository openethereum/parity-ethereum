use fetch::{Client as FetchClient, Fetch};
use ethkey::Public;
use super::{KeyServer, ServerKeyGenerator, DocumentKeyServer, MessageSigner};

pub struct HttpClient {
	base_url: String,
	fetch: FetchClient,
}

impl HttpClient {
	pub fn new(url: String) -> Self {
		SecretStoreClient {
			url: url,
		}
	}
}

impl ServerKeyGenerator for HttpClient {

}

impl DocumentKeyServer for HttpClient {
	fn store_document_key(&self, key_id: &ServerKeyId, signature: &RequestSignature, common_point: Public, encrypted_document_key: Public) -> Result<(), Error> {
		unimplemented!()
	}

	fn generate_document_key(&self, key_id: &ServerKeyId, signature: &RequestSignature, threshold: usize) -> Result<EncryptedDocumentKey, Error> {

	}

	fn restore_document_key(&self, key_id: &ServerKeyId, signature: &RequestSignature) -> Result<EncryptedDocumentKey, Error> {

	}

	fn restore_document_key_shadow(&self, _key_id: &ServerKeyId, _signature: &RequestSignature) -> Result<EncryptedDocumentKeyShadow, Error> {
		unimplemented!()
	}
}

impl MessageSigner for HttpClient {
	fn sign_message(&self, _key_id: &ServerKeyId, _signature: &RequestSignature, _message: MessageHash) -> Result<EncryptedMessageSignature, Error> {
		unimplemented!()
	}
}

impl KeyServer for HttpClient {}

	pub fn create_key(&self, key_id: H256, requester: Signature, threshold: u32) -> BoxFuture<Public, PrivateTransactionError> {
		let url = format!("{}/{}/{}/{}", self.base_url, key_id, requester, threshold);
		self.fetch.forget(self.fetch.fetch(&url)
			.map_err(|err| Error::Fetch(err))
			.and_then(move |mut response| {
				if !response.is_success() {
					return Err(Error::StatusCode(response.status().canonical_reason().unwrap_or("unknown")));
				}
				let mut result = String::new();
				response.read_to_string(&mut result)?;

				let value: Option<Value> = serde_json::from_str(&result).ok();
			})
		);
		/*let url = self.url.clone();
		url.join(&format!("{}/", key_id));
		url.join(&format!("{}/", requester));
		url.join(&format!("{}", threshold));

		let response = reqwest::get(url).unwrap();*/
	}
}
