#[test]
fn compile_fail() {
	let try_build = trybuild::TestCases::new();
	try_build.full_build(true);
	try_build.compile_fail("tests/compile-fail/*.rs");
}
