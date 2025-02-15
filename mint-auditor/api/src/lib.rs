// Copyright (c) 2018-2022 The MobileCoin Foundation

//! mobilecoind gRPC API.

use mc_util_uri::{Uri, UriScheme};

mod autogenerated_code {
    // Expose proto data types from included third-party/external proto files.
    pub use protobuf::well_known_types::Empty;

    // Needed due to how to the auto-generated code references the Empty message.
    pub mod empty {
        pub use super::Empty;
    }

    // Include the auto-generated code.
    include!(concat!(env!("OUT_DIR"), "/protos-auto-gen/mod.rs"));
}

pub use autogenerated_code::{mint_auditor::*, *};

pub type MintAuditorUri = Uri<MintAuditorScheme>;

/// Mint Auditor Uri Scheme
#[derive(Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct MintAuditorScheme {}
impl UriScheme for MintAuditorScheme {
    /// The part before the '://' of a URL.
    const SCHEME_SECURE: &'static str = "mint-auditor";
    const SCHEME_INSECURE: &'static str = "insecure-mint-auditor";

    /// Default port numbers
    const DEFAULT_SECURE_PORT: u16 = 7773;
    const DEFAULT_INSECURE_PORT: u16 = 7774;
}
