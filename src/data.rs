// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.1.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

pub use immutable_data::ImmutableData;
pub use priv_appendable_data::PrivAppendableData;
pub use pub_appendable_data::PubAppendableData;
use std::fmt::{self, Debug, Formatter};
pub use structured_data::StructuredData;
use xor_name::XorName;
use rust_sodium::crypto::sign;

/// A signing key with no matching private key. Passing ownership to it will make a chunk
/// effectively immutable.
pub const NO_OWNER_PUB_KEY: sign::PublicKey = sign::PublicKey([0; sign::PUBLICKEYBYTES]);

// Returns whether the signature is valid. It explicitly considers any signature for
// `NO_OWNER_PUB_KEY` invalid.
pub fn verify_detached(sig: &sign::Signature, data: &[u8], pub_key: &sign::PublicKey) -> bool {
    *pub_key != NO_OWNER_PUB_KEY && sign::verify_detached(sig, data, pub_key)
}

/// This is the data types routing handles in the public interface
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, RustcEncodable, RustcDecodable)]
pub enum Data {
    /// `StructuredData` data type.
    Structured(StructuredData),
    /// `ImmutableData` data type.
    Immutable(ImmutableData),
    /// `PubAppendableData` data type.
    PubAppendable(PubAppendableData),
    /// `PrivAppendableData` data type.
    PrivAppendable(PrivAppendableData),
}

impl Data {
    /// Return data name.
    pub fn name(&self) -> &XorName {
        match *self {
            Data::Structured(ref data) => data.name(),
            Data::Immutable(ref data) => data.name(),
            Data::PubAppendable(ref data) => data.name(),
            Data::PrivAppendable(ref data) => data.name(),
        }
    }

    /// Return data identifier.
    pub fn identifier(&self) -> DataIdentifier {
        match *self {
            Data::Structured(ref data) => data.identifier(),
            Data::Immutable(ref data) => data.identifier(),
            Data::PubAppendable(ref data) => data.identifier(),
            Data::PrivAppendable(ref data) => data.identifier(),
        }
    }

    /// Validate data size.
    pub fn validate_size(&self) -> bool {
        match *self {
            Data::Immutable(ref data) => data.validate_size(),
            Data::PrivAppendable(ref data) => data.validate_size(),
            Data::PubAppendable(ref data) => data.validate_size(),
            Data::Structured(ref data) => data.validate_size(),
        }
    }
}

#[derive(Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, RustcEncodable, RustcDecodable)]
/// An identifier to address a data chunk.
pub enum DataIdentifier {
    /// Data request, (Identifier, TypeTag) pair for name resolution, for StructuredData.
    Structured(XorName, u64),
    /// Data request, (Identifier), for `ImmutableData`.
    Immutable(XorName),
    /// Request for public appendable data.
    PubAppendable(XorName),
    /// Request for private appendable data.
    PrivAppendable(XorName),
}

impl Debug for Data {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            Data::Structured(ref data) => data.fmt(formatter),
            Data::Immutable(ref data) => data.fmt(formatter),
            Data::PubAppendable(ref data) => data.fmt(formatter),
            Data::PrivAppendable(ref data) => data.fmt(formatter),
        }
    }
}

impl DataIdentifier {
    /// DataIdentifier name.
    pub fn name(&self) -> &XorName {
        match *self {
            DataIdentifier::Structured(ref name, _) |
            DataIdentifier::Immutable(ref name) |
            DataIdentifier::PubAppendable(ref name) |
            DataIdentifier::PrivAppendable(ref name) => name,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use rust_sodium::crypto::hash::sha256;
    use rust_sodium::crypto::sign;
    use super::*;
    use xor_name::XorName;

    #[test]
    fn data_name() {
        // name() resolves correctly for StructuredData
        let keys = sign::gen_keypair();
        let owner_keys = vec![keys.0];
        match StructuredData::new(0,
                                  rand::random(),
                                  0,
                                  vec![],
                                  owner_keys.clone(),
                                  vec![],
                                  Some(&keys.1)) {
            Ok(structured_data) => {
                assert_eq!(structured_data.clone().name(),
                           Data::Structured(structured_data.clone()).name());
                assert_eq!(DataIdentifier::Structured(*structured_data.name(),
                                                      structured_data.get_type_tag()),
                           structured_data.identifier());
            }
            Err(error) => panic!("Error: {:?}", error),
        }


        // name() resolves correctly for ImmutableData
        let value = "immutable data value".to_owned().into_bytes();
        let immutable_data = ImmutableData::new(value);
        assert_eq!(immutable_data.name(),
                   Data::Immutable(immutable_data.clone()).name());
        assert_eq!(immutable_data.identifier(),
                   DataIdentifier::Immutable(*immutable_data.name()));
    }

    #[test]
    fn data_request_name() {
        let name = XorName(sha256::hash(&[]).0);

        // name() resolves correctly for StructuredData
        let tag = 0;
        assert_eq!(&name, DataIdentifier::Structured(name, tag).name());

        // name() resolves correctly for ImmutableData
        assert_eq!(&name, DataIdentifier::Immutable(name).name());
    }
}
