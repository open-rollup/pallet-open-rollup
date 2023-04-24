//  Copyright 2022 Open Rollup Lab
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

pub trait Verifier {
	/// Verify zk-program execution
	/// As one zk-program, should use old_state_root as the public inputs,
	/// user txs(L1_operations and L2 transactions) and state tree as secret inputs,
	/// the outputs of the zk-program's execution should include new_state_root,
	/// operations and l1_operations_pos (the number of the l1_operations included)
	fn vefify(
		program_hash: &[u8],
		old_state_root: &[u8],
		proof: &[u8],
		outputs: &[u8],
	) -> Result<(), ()>;
}

mod miden_verifier;

pub use miden_verifier::MidenVerifier;

/// One Fake verifier for testing.
pub struct FakeVerifier;
impl Verifier for FakeVerifier {
	#[allow(unused_variables)]
	fn vefify(
		program_hash: &[u8],
		proof: &[u8],
		old_state_root: &[u8],
		outputs: &[u8],
	) -> Result<(), ()> {
		Ok(())
	}
}
