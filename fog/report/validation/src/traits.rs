use crate::ingest_report::Error as IngestReportError;
use core::fmt::Debug;
use displaydoc::Display;
use mc_account_keys::PublicAddress;
use mc_crypto_keys::RistrettoPublic;

#[cfg(any(test, feature = "automock"))]
use mockall::*;

/// Class that can resolve a public address to a fully-validated fog public key structure,
/// including the pubkey expiry data from the report server.
#[cfg_attr(any(test, feature = "automock"), automock)]
pub trait FogPubkeyResolver {
    /// Fetch and validate a fog public key, given a recipient's public address
    fn get_fog_pubkey(
        &self,
        recipient: &PublicAddress,
    ) -> Result<FullyValidatedFogPubkey, FogPubkeyError>;
}

/// Represents a fog public key validated to use for creating encrypted fog hints.
/// This object should be constructed only when the IAS report has been validated,
/// and the chain of trust from the connection has been validated, and the
/// the fog user's fog_authority_fingerprint_sig over the fingerprints in the signature chain
/// has been validated for at least one fingerprint.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct FullyValidatedFogPubkey {
    /// The ristretto curve point which was extracted from the IAS report additional data
    /// after validation. This is the encryption key used to create encrypted fog hints.
    /// The corresponding private key lives only in SGX ingest nodes.
    pub pubkey: RistrettoPublic,
    /// The pubkey_expiry value is the latest block that fog-service promises
    /// that is valid to encrypt fog hints using this key for.
    /// The client should obey this limit by not setting tombstone block for a
    /// transaction larger than this limit if the fog pubkey is used.
    pub pubkey_expiry: u64,
}

/// An error that can occur when trying to get a validated fog pubkey from the
/// FogResolver object
/// TODO: Add x509 errors, user-sig check errors, etc. here
#[derive(Display, Debug)]
pub enum FogPubkeyError {
    /// No matching reports response for url = {0}
    NoMatchingReportResponse(String),
    /// No matching report id for url = {0}, report_id = {1}
    NoMatchingReportId(String, String),
    /// Address has no fog_report_url, cannot fetch fog pubkey
    NoFogReportUrl,
    /// Ingest report deserialization error: {0},
    Deserialization(mc_util_serial::decode::Error),
    /// Ingest report verification error: {0}
    IngestReport(IngestReportError),
}

impl From<IngestReportError> for FogPubkeyError {
    fn from(src: IngestReportError) -> Self {
        Self::IngestReport(src)
    }
}

impl From<mc_util_serial::decode::Error> for FogPubkeyError {
    fn from(src: mc_util_serial::decode::Error) -> Self {
        Self::Deserialization(src)
    }
}
