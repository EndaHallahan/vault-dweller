use vault_dweller::{ VaultIndex, VaultItem };
use std::env;
use std::path::{ PathBuf };

fn get_vault_path() -> PathBuf {
	let mut p = env::current_dir().unwrap();
	p.push("tests\\TestVault");
	p
}

#[test]
fn vault_index_can_be_created() {
	let vi = VaultIndex::new(None, true);
	assert_eq!(vi.is_ok(), true);
}

#[test]
fn vault_index_can_read_vault() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str(), true);
	assert_eq!(vi.is_ok(), true);
}

#[test]
fn vault_index_invalid_vault_path() {
	let vi = VaultIndex::new(Some("tests\\argabarga"), true);
	assert_eq!(vi.is_err(), true);
}

#[test]
fn vault_index_can_get_item() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
	let fc = vi.get_item("This is the Test Vault");
	assert_eq!(fc.is_some(), true);
	match fc.unwrap() {
		VaultItem::Note(_) => {},
		_ => {panic!("Item wasn't a note!");}
	}
	//println!("{:?}", fc.unwrap());
}

#[test]
fn vault_index_can_get_item_dir_path() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
	let fc = vi.get_item("Folder A/Lorem Ipsum");
	assert_eq!(fc.is_some(), true);
	//println!("{:?}", fc.unwrap());
}

#[test]
fn vault_index_invalid_file_get_path() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
	let fc = vi.get_item("Folder Z/Recarm");
	assert_eq!(fc.is_none(), true);
}

#[test]
fn vault_index_can_get_note_contents() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
	let fc = vi.get_note_contents("This is the Test Vault");
	assert_eq!(fc.is_ok(), true);
	//println!("{:?}", fc.unwrap());
}

#[test]
fn vault_index_can_get_note_contents_dir_path() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
	let fc = vi.get_note_contents("Folder A/Lorem Ipsum");
	assert_eq!(fc.is_ok(), true);
	//println!("{:?}", fc.unwrap());
}

#[test]
fn vault_index_invalid_file_contents_get_path() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
	let fc = vi.get_note_contents("Folder Z/Recarm");
	assert_eq!(fc.is_err(), true);
}

#[test]
fn vault_index_can_get_item_as_json() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
	let fc = vi.get_item("Folder A/Lorem Ipsum").expect("Couldn't get file!");
	match fc {
		VaultItem::Note(n) => {
			let _json: String = n.as_json();
			//println!("{:?}", json);
		},
		_ => {panic!("Item wasn't a note!");}
	}
	
}

#[test]
fn vault_index_can_get_item_properties_as_json() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
	let fc = vi.get_item("Folder A/Lorem Ipsum").expect("Couldn't get file!");
	match fc {
		VaultItem::Note(n) => {
			let _json: String = n.properties_as_json();
			//println!("{:?}", json);
		},
		_ => {panic!("Item wasn't a note!");}
	}
}

#[test]
fn vault_index_can_dataview() {
	let p = get_vault_path();
	let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
	vi.query("LIST FROM #Lorem AND (#Ipsum OR #test)");
}