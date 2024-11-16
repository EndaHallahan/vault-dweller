use vault_dweller::VaultIndex;
use std::env;
use std::path::{ PathBuf };

fn get_vault_path() -> PathBuf {
	let mut p = env::current_dir().unwrap();
	p.push("tests\\TestVault");
	p
}

#[test]
fn vault_index_can_be_created() {
	let vi = VaultIndex::new(None);
	assert_eq!(vi.is_ok(), true);
}

#[test]
fn vault_index_can_read_vault() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str());
	assert_eq!(vi.is_ok(), true);
}

#[test]
fn vault_index_invalid_vault_path() {
	let vi = VaultIndex::new(Some("tests\\argabarga"));
	assert_eq!(vi.is_err(), true);
}

#[test]
fn vault_index_can_get_file() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str()).expect("Couldn't make Vault Index!");
	let fc = vi.get_file("This is the Test Vault");
	assert_eq!(fc.is_some(), true);
	//println!("{:?}", fc.unwrap());
}

#[test]
fn vault_index_can_get_file_dir_path() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str()).expect("Couldn't make Vault Index!");
	let fc = vi.get_file("Folder A/Lorem Ipsum");
	assert_eq!(fc.is_some(), true);
	//println!("{:?}", fc.unwrap());
}

#[test]
fn vault_index_invalid_file_get_path() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str()).expect("Couldn't make Vault Index!");
	let fc = vi.get_file("Folder Z/Recarm");
	assert_eq!(fc.is_none(), true);
}

#[test]
fn vault_index_can_get_file_contents() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str()).expect("Couldn't make Vault Index!");
	let fc = vi.get_file_contents("This is the Test Vault");
	assert_eq!(fc.is_ok(), true);
	//println!("{:?}", fc.unwrap());
}

#[test]
fn vault_index_can_get_file_contents_dir_path() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str()).expect("Couldn't make Vault Index!");
	let fc = vi.get_file_contents("Folder A/Lorem Ipsum");
	assert_eq!(fc.is_ok(), true);
	//println!("{:?}", fc.unwrap());
}

#[test]
fn vault_index_invalid_file_contents_get_path() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str()).expect("Couldn't make Vault Index!");
	let fc = vi.get_file_contents("Folder Z/Recarm");
	assert_eq!(fc.is_err(), true);
}

#[test]
fn vault_index_can_get_file_as_json() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str()).expect("Couldn't make Vault Index!");
	let fc = vi.get_file("Folder A/Lorem Ipsum").expect("Couldn't get file!");
	let _json: String = fc.as_json();
	//println!("{:?}", json);
}

#[test]
fn vault_index_can_get_file_properties_as_json() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str()).expect("Couldn't make Vault Index!");
	let fc = vi.get_file("Folder A/Lorem Ipsum").expect("Couldn't get file!");
	let _json: String = fc.properties_as_json();
	//println!("{:?}", json);
}