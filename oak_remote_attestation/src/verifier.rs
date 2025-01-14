//
// Copyright 2023 The Project Oak Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use crate::proto::oak::session::v1::{AttestationEndorsement, AttestationEvidence};
use alloc::vec::Vec;

/// Reference values used by the verifier to appraise the attestation evidence.
/// <https://www.rfc-editor.org/rfc/rfc9334.html#name-reference-values>
pub struct ReferenceValue {
    pub binary_hash: Vec<u8>,
}

/// A trait implementing the functionality of a verifier that appraises the attestation evidence and
/// produces an attestation result.
/// <https://www.rfc-editor.org/rfc/rfc9334.html#name-verifier>
pub trait AttestationVerifier: Clone + Send + Sync {
    /// Verify that the provided evidence was endorsed and contains specified reference values.
    fn verify(
        evidence: &AttestationEvidence,
        endorsement: &AttestationEndorsement,
        reference_value: &ReferenceValue,
    ) -> anyhow::Result<()>;
}

/// An instance of [`AttestationVerifier`] that succeeds iff the provided attestation is empty.
///
/// Useful when no attestation is expected to be genereated by the other side of a remotely
/// attested connection.
#[derive(Clone)]
pub struct InsecureAttestationVerifier;

impl AttestationVerifier for InsecureAttestationVerifier {
    fn verify(
        evidence: &AttestationEvidence,
        _endorsement: &AttestationEndorsement,
        _reference_value: &ReferenceValue,
    ) -> anyhow::Result<()> {
        // We check that the attestation report is empty in order to avoid accidentally ignoring a
        // real attestation from the other side, although in principle a more lenient
        // implementation of this struct could be used that always ignores also non-empty
        // attestations.
        if evidence.attestation.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "expected empty attestation report, got {:?}",
                evidence.attestation
            ))
        }
    }
}
