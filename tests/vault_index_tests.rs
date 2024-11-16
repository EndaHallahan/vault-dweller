use vault_dweller::VaultIndex;
use std::env;

#[test]
fn vault_index_can_be_created() {
	let vi = VaultIndex::new(None);
}

#[test]
fn vault_index_can_read_vault() {
	let mut p = env::current_dir().unwrap();
	p.push("tests\\TestVault");
	let vi = VaultIndex::new(p.to_str());
	assert_eq!(vi.is_ok(), true);
}

#[test]
fn vault_index_invalid_filepath_test() {
	let vi = VaultIndex::new(Some("tests\\argabarga"));
	assert_eq!(vi.is_err(), true);
}