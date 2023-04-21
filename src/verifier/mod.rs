pub trait Verifier {
	// old_state_root as public inputs, and user txs(L1_operations and L2 transactions) and old
	// state tree as secret inputs outputs should include new_state_root, operations and
	// l1_operations_pos
	fn vefify(
		program_hash: &[u8],
		old_state_root: &[u8],
		proof: &[u8],
		outputs: &[u8],
	) -> Result<(), ()>;
}

mod miden_verifier;

pub use miden_verifier::MidenVerifier;

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
