use serde::{Deserialize, Serialize};

use openidconnect::core::{
    CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClientAuthMethod, CoreGrantType,
    CoreJsonWebKey, CoreJsonWebKeyType, CoreJsonWebKeyUse, CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm, CoreJwsSigningAlgorithm, CoreResponseMode, CoreResponseType,
    CoreSubjectIdentifierType,
};
use openidconnect::{AdditionalProviderMetadata, CsrfToken, Nonce, ProviderMetadata};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RevocationEndpointProviderMetadata {
    pub revocation_endpoint: String,
}
impl AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}
pub type GoogleProviderMetadata = ProviderMetadata<
    RevocationEndpointProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;
#[derive(Serialize, Deserialize)]
pub struct OidcSession {
    pub nonce: Nonce,
    pub csrf_token: CsrfToken,
}
