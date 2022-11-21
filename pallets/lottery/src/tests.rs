use crate::mock::{new_test_ext, run_to_block, System};

#[test]
fn fake_test_example() {
	new_test_ext().execute_with(|| {
		assert_eq!(System::block_number(), 1);
		run_to_block(5);
		assert_eq!(System::block_number(), 5);
	});
}
