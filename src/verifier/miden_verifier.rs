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

use super::Verifier;
use miden::{
	math::Felt,
	utils::{ByteReader, Deserializable, SliceReader},
	Digest, ExecutionProof, Kernel, ProgramInfo, StackInputs, StackOutputs,
};
use sp_std::vec::Vec;

/// Verifier for Miden program.
///
/// <https://github.com/0xPolygonMiden/miden-vm/tree/main/miden>
pub struct MidenVerifier;
impl Verifier for MidenVerifier {
	/// Verify miden program execution
	///
	/// The `old_state_root` as the public inputs.
	/// The stack included in `outputs` include new_state_root, operations and l1_operations_pos.
	fn vefify(
		program_hash: &[u8],
		old_state_root: &[u8],
		proof: &[u8],
		outputs: &[u8],
	) -> Result<(), ()> {
		// check program_hash is valid.
		let program_hash = Digest::read_from_bytes(program_hash).map_err(|_| ())?;
		let program_info = ProgramInfo::new(program_hash, Kernel::default());
		let proof = ExecutionProof::from_bytes(proof).map_err(|_| ())?;

		// stack inputs deserialize from old_state_root
		let miden_inputs = raw_inputs_to_stack_inputs(old_state_root).map_err(|_| ())?;

		// outputs deserialize.
		let miden_outputs;
		{
			let mut outputs_reader = SliceReader::new(outputs);
			let stack_len = outputs_reader.read_u32().map_err(|_| ())?;
			let mut stack = Vec::new();
			for _ in 0..stack_len {
				stack.push(Felt::new(outputs_reader.read_u64().map_err(|_| ())?))
			}

			let overflow_addrs_lens = outputs_reader.read_u32().map_err(|_| ())?;
			let mut overflow_addrs = Vec::new();
			for _ in 0..overflow_addrs_lens {
				overflow_addrs.push(Felt::new(outputs_reader.read_u64().map_err(|_| ())?))
			}

			miden_outputs = StackOutputs::from_elements(stack, overflow_addrs);
		}

		// println!("program_info: {:?}", program_info);
		// println!("StackOutputs: {:?}", miden_outputs);

		miden::verify(program_info, miden_inputs, miden_outputs, proof).map_err(|_| ())?;

		Ok(())
	}
}

/// Convert bytes to Miden's `StackInputs`.
fn raw_inputs_to_stack_inputs(raw_data: &[u8]) -> Result<StackInputs, ()> {
	if raw_data.len() != 32 {
		return Err(())
	}

	let miden_inputs;
	{
		let mut inputs_reader = SliceReader::new(raw_data);
		let mut stack = Vec::new();
		for _ in 0..4 {
			stack.push(inputs_reader.read_u64().map_err(|_| ())?)
		}
		miden_inputs = StackInputs::try_from_values(stack).map_err(|_| ())?;
	}

	Ok(miden_inputs)
}

#[cfg(test)]
mod tests {
	use super::*;
	use miden::{prove, utils::Serializable, Assembler, MemAdviceProvider, ProofOptions};
	use sp_runtime::testing::H256;

	/// Test Miden Verifier should work.
	#[test]
	fn it_works() {
		let assembler = Assembler::default();
		let program = assembler.compile("begin push.3 push.5 add end").unwrap();

		let old_state_root = H256::repeat_byte(1);

		let inputs = raw_inputs_to_stack_inputs(old_state_root.as_bytes()).unwrap();

		let (outputs, proof) =
			prove(&program, inputs, MemAdviceProvider::default(), ProofOptions::default()).unwrap();

		// println!("program_info: {:?}", ProgramInfo::from(program.clone()));
		// println!("StackOutputs: {:?}", outputs);

		assert_eq!(*outputs.stack().first().unwrap(), 8);
		assert_eq!(
			MidenVerifier::vefify(
				&program.hash().as_bytes(),
				old_state_root.as_bytes(),
				&proof.to_bytes(),
				&outputs.to_bytes()
			),
			Ok(())
		);
	}
}
