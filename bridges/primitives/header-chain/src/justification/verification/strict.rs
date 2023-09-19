// Copyright 2019-2023 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Logic for checking if GRANDPA Finality Proofs are valid and optimal.

use crate::justification::{
	verification::{Error, JustificationVerifier, PrecommitError},
	GrandpaJustification,
};

use crate::justification::verification::{
	IterationFlow, JustificationVerificationContext, SignedPrecommit,
};
use sp_consensus_grandpa::AuthorityId;
use sp_runtime::traits::Header as HeaderT;
use sp_std::collections::btree_set::BTreeSet;

/// Verification callbacks that reject all unknown, duplicate or redundant votes.
struct StrictJustificationVerifier {
	votes: BTreeSet<AuthorityId>,
}

impl<Header: HeaderT> JustificationVerifier<Header> for StrictJustificationVerifier {
	fn process_redundant_vote(
		&mut self,
		_precommit_idx: usize,
	) -> Result<IterationFlow, PrecommitError> {
		Err(PrecommitError::RedundantAuthorityVote)
	}

	fn process_known_authority_vote(
		&mut self,
		_precommit_idx: usize,
		signed: &SignedPrecommit<Header>,
	) -> Result<IterationFlow, PrecommitError> {
		if self.votes.contains(&signed.id) {
			// There's a lot of code in `validate_commit` and `import_precommit` functions
			// inside `finality-grandpa` crate (mostly related to reporting equivocations).
			// But the only thing that we care about is that only first vote from the
			// authority is accepted
			return Err(PrecommitError::DuplicateAuthorityVote)
		}

		Ok(IterationFlow::Run)
	}

	fn process_unknown_authority_vote(
		&mut self,
		_precommit_idx: usize,
	) -> Result<(), PrecommitError> {
		Err(PrecommitError::UnknownAuthorityVote)
	}

	fn process_unrelated_ancestry_vote(
		&mut self,
		_precommit_idx: usize,
	) -> Result<IterationFlow, PrecommitError> {
		Err(PrecommitError::UnrelatedAncestryVote)
	}

	fn process_invalid_signature_vote(
		&mut self,
		_precommit_idx: usize,
	) -> Result<(), PrecommitError> {
		Err(PrecommitError::InvalidAuthoritySignature)
	}

	fn process_valid_vote(&mut self, signed: &SignedPrecommit<Header>) {
		self.votes.insert(signed.id.clone());
	}

	fn process_redundant_votes_ancestries(
		&mut self,
		_redundant_votes_ancestries: BTreeSet<Header::Hash>,
	) -> Result<(), Error> {
		Err(Error::RedundantVotesAncestries)
	}
}

/// Verify that justification, that is generated by given authority set, finalizes given header.
pub fn verify_justification<Header: HeaderT>(
	finalized_target: (Header::Hash, Header::Number),
	context: &JustificationVerificationContext,
	justification: &GrandpaJustification<Header>,
) -> Result<(), Error> {
	let mut verifier = StrictJustificationVerifier { votes: BTreeSet::new() };
	verifier.verify_justification(finalized_target, context, justification)
}
