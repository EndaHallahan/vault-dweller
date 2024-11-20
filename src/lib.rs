// #![warn(missing_docs)]
//! # Vault Dweller
//!
//! Vault Dweller makes it more convenient to work with 
//! [Obsidian](https://obsidian.md/) vaults programmatically.
//!
//! ## Features
//!
//! Vault Dweller provides a struct, [`VaultIndex`], which acts as a 
//! collection of files and folders in a vault. Files can be accessed
//! by name or local path, the same way notes can be linked in Obsidian.
//!
//! Files are represented as [`NoteItem`]s, and contain their own metadata
//! (front matter, tags). They can also fetch their own contents from the 
//! disk.
//!
//! ## Examples
//!
//! ```rust
//! use vault_dweller::VaultIndex;
//! use std::env;
//!
//! let mut p = env::current_dir().unwrap();
//! p.push("tests\\TestVault");
//! let vi = VaultIndex::new(p.to_str(), true).unwrap();
//! let fc = vi.get_item("This is the Test Vault");
//! assert_eq!(vec!["test".to_string()], fc.unwrap().unwrap_note().tags);
//! ```

use std::io;
use std::fs::{ self };
use std::path::{ PathBuf };
use std::collections::HashMap;
use chrono::{ DateTime, Utc, serde::ts_seconds };
use indexmap::{ IndexMap };
use regex::Regex;
use yaml_rust::{ YamlLoader, Yaml };
use serde::{ Deserialize, Serialize };

mod dataview;

/// Represents a property in a note's front matter.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Property {
    Text(String),
    Number(f64),
    Checkbox(bool),
    List(Vec<Property>),
    #[serde(with = "ts_seconds")]
    Date(DateTime<Utc>),
    Unknown,
}

#[derive(Debug)]
pub enum VaultItem<'a> {
    Note(&'a NoteItem),
    File(&'a FileItem),
}
impl <'a> VaultItem<'a> {
    pub fn unwrap_note(&self) -> &NoteItem {
        return match self {
            VaultItem::Note(n) => n,
            _ => {panic!("Called unwrap_note on not a note!");}
        }
    }

    pub fn unwrap_file(&self) -> &FileItem {
        return match self {
            VaultItem::File(n) => n,
            _ => {panic!("Called unwrap_file on not a file!");}
        }
    }
}

#[derive(Debug, Clone)]
pub enum ItemType {
    File,
    Folder,
    Note,
    Root
}

#[derive(Debug)]
pub struct TreeNode {
    pub name: String,
    pub index: usize,
    pub item: ItemType,
    pub children: Vec<usize>,
    pub depth: u32,
}
impl TreeNode {
    fn new(index: usize, item: ItemType, name: String, depth: u32) -> Self {
        Self {
            name,
            index,
            item,
            children: vec![],
            depth
        }
    }
}

#[derive(Debug)]
pub struct Tree {
    arena: Vec<TreeNode>,
}
impl Tree {
    pub fn new() -> Self {
        let mut new_tree = Self {
            arena: vec![],
        };
        new_tree.add_node("root".to_string(), ItemType::Root, 0);
        new_tree
    }
    pub fn get_root(&self) -> &TreeNode {
        &self.arena[0]
    }

    pub fn get_node(&self, index: usize) -> Option<&TreeNode> {
        self.arena.get(index)
    }

    pub fn get_node_mut(&mut self, index: usize) -> Option<&mut TreeNode> {
        self.arena.get_mut(index)
    }

    pub fn has_node(&self, index: usize) -> bool {
         self.arena.len() > index
    }

    pub fn add_child(&mut self, parent: usize, name: String, item: ItemType) -> Option<usize> {
        if !self.has_node(parent) {
            return None;
        }
        let idx = self.add_node(name, item, 0);
        let mut depth: u32 = 0;
        if let Some(parent_node) = self.get_node_mut(parent) {
            parent_node.children.push(idx);
            depth = parent_node.depth
        } else {
            return None;
        }  
        self.arena[idx].depth = depth + 1;
        return Some(idx);
    }

    pub fn as_flat_vec(&self, node_index: usize) -> Vec<&TreeNode> {
        let mut nodes: Vec<&TreeNode> = vec![];
        let node = self.get_node(node_index).expect("Couldn't find node of that index!");
        let mut children: Vec<&TreeNode> = vec![];
        for child in &node.children {
            let mut child_children: Vec<&TreeNode>  = self.as_flat_vec(*child);
            children.append(&mut child_children);
        }
        nodes.push(node);
        nodes.append(&mut children);
        nodes
    }

    fn add_node(&mut self, name: String, item: ItemType, depth: u32) -> usize {
        let idx = self.arena.len();
        self.arena.push(TreeNode::new(idx, item, name, depth));
        idx
    }
}

#[derive(Debug)]
enum FileFolder {
    File(FileItem),
    Folder(FolderItem),
    Note(NoteItem),
}

/// Represents a folder in the Vault.
///
/// Note that folders cannot get their children, and that this struct
/// exists more as a way of referencing that a folder is there than
/// as a way of interacting with the vault. If you need to do that,
/// you may want to work with the `local_path` field on [`NoteItem`] 
/// instead.
#[derive(Debug, Serialize, Deserialize)]
pub struct FolderItem {
    pub name: String,
    pub path: PathBuf,
    pub local_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub file_type: String,
    pub path: PathBuf,
    pub local_path: PathBuf,
}

/// Represents a note in the Vault.
///
/// The `properties` field represents the properties defined in a note's
/// front matter, as a HashMap of [`Property`] enums.
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteItem {
    pub name: String,
    pub file_type: String,
    pub path: PathBuf,
    pub local_path: PathBuf,
    pub properties: HashMap<String, Property>,
    pub tags: Vec<String>,
}
impl NoteItem {
    /// Returns a representation of this struct as a json string.
    pub fn as_json(&self) -> String {
        serde_json::to_string(self).expect(&format!("Couldn't parse NoteItem {:?} into JSON!", self.name))
    }
    /// Returns a representation of this struct's `properties` 
    /// field as a json string.
    pub fn properties_as_json(&self) -> String {
        serde_json::to_string(&self.properties).expect(&format!("Couldn't parse NoteItem {:?} properties into JSON!", self.name))
    }
    /// Retrieves the contents of the note from the disk.
    pub fn get_contents(&self) -> Result<String, io::Error> {
        fs::read_to_string(&self.path)
    }
}

/// Represents everything in a vault.
#[derive(Debug)]
pub struct VaultIndex {
    pub name: String,
    pub path: Option<PathBuf>,
    pub notes: IndexMap<String, NoteItem>,
    pub files: IndexMap<String, FileItem>,
    pub folders: Vec<FolderItem>,
    pub filepath_ref: IndexMap<String, String>,
    pub tags: IndexMap<String, Vec<String>>,
    pub properties: Vec<String>,
    pub tree: Tree,
}

impl VaultIndex {
    /// Creates a new [`VaultIndex`], given the path to a vault as a string.
    /// The path is wrapped in an `Option`, and you may supply `None` if you
    /// do not want to generate a [`VaultIndex`] from an existing vault.
    pub fn new(path_to_vault: Option<&str>, include_obsidian_folder: bool) -> Result<Self, io::Error> {
        let mut name: String = Default::default();
        let mut path: PathBuf = PathBuf::new();
        let mut notes: IndexMap<String, NoteItem> = IndexMap::new();
        let mut files: IndexMap<String, FileItem> = IndexMap::new();
        let mut folders: Vec<FolderItem> = vec![];
        let mut filepath_ref: IndexMap<String, String> = IndexMap::new();
        let mut tags: IndexMap<String, Vec<String>> = Default::default();
        let mut properties: Vec<String> = vec![];
        let mut tree: Tree = Tree::new();
        if let Some(vault_path) = path_to_vault {
            let p = PathBuf::from(vault_path);
            name = p.file_name().unwrap().to_str().unwrap().to_owned();
            tree.arena[0].name = name.clone();
            path = p.clone();
            if !p.is_dir() {
                return Err(io::Error::new(io::ErrorKind::NotFound, "The path specified either could not be found, could not be accessed, or was not a directory."));
            }

            let file_collection = Self::recursive_generate_filefolders(&p, &p, include_obsidian_folder, &mut tree, 0);
            
            for file in file_collection {
                match file {
                    FileFolder::Note(fi) => {
                        filepath_ref.insert(fi.local_path.to_str().unwrap().to_string().clone(), fi.name.clone());
                        for tag in &fi.tags {
                            if let Some(tag_list) = tags.get_mut(tag) {
                                tag_list.push(fi.name.clone());
                            } else {
                                tags.insert(tag.clone(), vec![fi.name.clone()]);
                            }
                        }
                        for key in fi.properties.keys() {
                            properties.push(key.clone());
                        }
                        notes.insert(fi.name.clone(), fi);
                    }
                    FileFolder::File(fi) => {
                        filepath_ref.insert(fi.local_path.to_str().unwrap().to_string().clone(), fi.name.clone());
                        files.insert(fi.name.clone(), fi); 
                    },
                    FileFolder::Folder(fi) => {
                        folders.push(fi);
                    },
                }
            }
            
            /*
            println!("\n==== NAME ====");
            println!("{:?}", name);
            */
            /*
            println!("\n==== NOTES ====");
            println!("{:?}", notes);
            */
            /*
            println!("\n==== FILES ====");
            println!("{:?}", files);
            */
            /*
            println!("\n==== FILE PATHS ====");
            println!("{:?}", filepath_ref);
            */
            /*
            println!("\n==== FOLDERS ====");
            println!("{:?}", folders);
            */
            /*
            println!("\n==== TAGS ====");
            println!("{:?}", tags);
            */
            /*
            println!("\n==== PROPERTIES ====");
            println!("{:?}", properties);
            */
            /*
            println!("\n==== TREE ====");
            println!("{:?}", tree);
            */
        }

        properties.dedup();

        let vi = VaultIndex {
            name,
            path: Some(path),
            notes,
            files,
            folders,
            filepath_ref,
            tags,
            properties,
            tree,
        };

        Ok(vi)
    }

    /// Retrieves a [`NoteItem`] from the [`VaultIndex`] by name or local
    /// path. Returns `None` if there was no file matching that name/path
    /// in the index.
    ///
    /// ```rust
    /// use vault_dweller::VaultIndex;
    /// use std::env;
    ///
    /// let mut p = env::current_dir().unwrap();
    /// p.push("tests\\TestVault");
    /// let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
    /// /* Name */
    /// let fa = vi.get_item("This is the Test Vault");
    /// /* Local Path */
    /// let fb = vi.get_item("Folder A/Lorem Ipsum");
    /// ```
    pub fn get_item(&self, local_path: &str) -> Option<VaultItem> {
        let mut adj_local_path: &str = &local_path.replace("/", "\\");
        match adj_local_path.find("\\") {
            Some(_) =>  {
                if let Some(p) = self.filepath_ref.get(adj_local_path) {
                    adj_local_path = &p;
                } else {
                    return None;
                }
            },
            None => {
                adj_local_path = local_path;
            },
        };
        if let Some(note) = self.notes.get(adj_local_path) {
            return Some(VaultItem::Note(note));
        } else if let Some(file) = self.files.get(adj_local_path) {
            return Some(VaultItem::File(file));
        } else {
            return None;
        }
    }

    pub fn get_note(&self, local_path: &str) -> Option<&NoteItem> {
        let mut adj_local_path: &str = &local_path.replace("/", "\\");
        match adj_local_path.find("\\") {
            Some(_) =>  {
                if let Some(p) = self.filepath_ref.get(adj_local_path) {
                    adj_local_path = &p;
                } else {
                    return None;
                }
            },
            None => {
                adj_local_path = local_path;
            },
        };

        self.notes.get(adj_local_path)
    }

    /// Retrieves a note's contents by name or local path as a String. 
    /// It will return an Error if the file cannot be found or cannot be opened.
    ///
    /// ```rust
    /// use vault_dweller::VaultIndex;
    /// use std::env;
    ///
    /// let mut p = env::current_dir().unwrap();
    /// p.push("tests\\TestVault");
    /// let vi = VaultIndex::new(p.to_str(), true).expect("Couldn't make Vault Index!");
    /// /* Name */
    /// let fa = vi.get_note_contents("This is the Test Vault");
    /// /* Local Path */
    /// let fb = vi.get_note_contents("Folder A/Lorem Ipsum");
    /// ```
    pub fn get_note_contents(&self, local_path: &str) -> Result<String, io::Error> {
        if let Some(entry) = self.get_item(local_path) {
            return match entry {
                VaultItem::Note(n) => n.get_contents(),
                _ => Err(io::Error::new(io::ErrorKind::Other, "Couldn't match local path!")),
            }
            
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "Couldn't match local path!"));
        }  
    }

    pub fn query(&self, in_query: &str) {
        dataview::to_view(in_query, &self)
    }

    fn recursive_generate_filefolders(dir_path: &PathBuf, vault_path: &PathBuf, include_obsidian_folder: bool, tree: &mut Tree, tree_parent: usize) -> Vec<FileFolder> {
        let mut out_filefolders: Vec<FileFolder> = vec![];
        let fpath = fs::read_dir(dir_path);
        match fpath {
            Ok(paths) => {
                for path in paths {
                    let child_file = path.unwrap();
                    if child_file.file_type().unwrap().is_dir() {
                        if !include_obsidian_folder && &child_file.path().file_name().unwrap().to_str().unwrap() == &".obsidian" {
                            continue;
                        }
                        out_filefolders.push(Self::generate_folder_item(&child_file.path(), vault_path).unwrap());
                        let idx = tree.add_child(tree_parent, child_file.path().file_name().unwrap().to_str().unwrap().to_owned(), ItemType::Folder).expect("Couldn't find parent in tree!");
                        let mut children_filepaths = Self::recursive_generate_filefolders(&child_file.path(), vault_path, include_obsidian_folder, tree, idx);
                        out_filefolders.append(&mut children_filepaths);
                    } else if child_file.path().extension().unwrap() == "md" {
                        tree.add_child(tree_parent, child_file.path().file_stem().unwrap().to_str().unwrap().to_owned(), ItemType::Note);
                        out_filefolders.push(Self::generate_note_item(&child_file.path(), vault_path).unwrap());
                    } else {
                        tree.add_child(tree_parent, child_file.path().file_name().unwrap().to_str().unwrap().to_owned(), ItemType::File);
                        out_filefolders.push(Self::generate_file_item(&child_file.path(), vault_path).unwrap());
                    }
                }
            },
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        out_filefolders
    }

    fn generate_folder_item(path: &PathBuf, vault_path: &PathBuf) -> Result<FileFolder, io::Error> {
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let local_path = path.strip_prefix(vault_path).unwrap().to_path_buf();
        let fi = FolderItem {
            name,
            path: path.to_path_buf(),
            local_path,
        };
        Ok(FileFolder::Folder(fi))
    }

    fn generate_file_item(path: &PathBuf, vault_path: &PathBuf) -> Result<FileFolder, io::Error> {
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let file_type = path.extension().unwrap().to_str().unwrap().to_owned();
        let local_path = path.strip_prefix(vault_path).unwrap().to_path_buf();
        let fi = FileItem {
            name,
            file_type,
            path: path.to_path_buf(),
            local_path,
        };
        Ok(FileFolder::File(fi))
    }

    fn tag_splitter(tag: String) -> Vec<String> {
        let mut out_tags: Vec<String> = vec![];
        let tag_slices: Vec<&str> = tag.split('/').collect();
        
        for i in 0..tag_slices.len() {
            let mut tag_string: String = Default::default();
            for j in 0..i+1 {
                if j != 0 {
                    tag_string.push_str("/");
                }
                tag_string.push_str(tag_slices[j]);
            }
            out_tags.push(tag_string);
        }

        out_tags
    }

    fn generate_note_item(path: &PathBuf, vault_path: &PathBuf) -> Result<FileFolder, io::Error> {
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let file_type = path.extension().unwrap().to_str().unwrap().to_owned();
        let mut local_path = path.strip_prefix(vault_path).unwrap().to_path_buf();
        local_path.set_extension("");

        // Figure out a way to remove these from the loop
        let tag_matcher = Regex::new(r"(\B#\S+)").expect("REGEX FAILED");
        let properties_matcher = Regex::new(r"(---[\w\W]*?---)").expect("REGEX FAILED");
        let codeblock_matcher = Regex::new(r"```[\w\W]*```").expect("REGEX FAILED");
        let inline_codeblock_matcher = Regex::new(r"[^\n\r`]+?`").expect("REGEX FAILED");

        let mut tags: Vec<String> = vec![];
        let mut properties: HashMap<String, Property> = Default::default();
        
        let file_contents = fs::read_to_string(path);

        match file_contents {
            Ok(cont) => {
                let mut adj_cont = codeblock_matcher.replace_all(&cont, "").to_string();
                adj_cont = inline_codeblock_matcher.replace_all(&adj_cont, "").to_string();
                //println!("{:?}", &cont);
                for (_, [tag]) in tag_matcher.captures_iter(&adj_cont).map(|c| c.extract()) {
                    let mut split_tags = Self::tag_splitter(tag.replace('#', ""));
                    tags.append(&mut split_tags);
                }
                tags.sort();
                tags.dedup();

                if let Some(ind) = adj_cont.find("---") {
                    if ind == 0 {
                        let properties_match = properties_matcher.captures(&adj_cont).unwrap();
                        properties = Self::generate_properties(properties_match.get(0).unwrap().as_str().replace("---", "").trim()).unwrap();

                    }
                }
            },
            Err(e) => {
                return Err(e);
            }
        }

        let fi = NoteItem {
            name,
            file_type,
            path: path.to_path_buf(),
            local_path,
            properties,
            tags,
       };
       Ok(FileFolder::Note(fi))
    }

    fn generate_properties(property_yaml: &str) -> Result<HashMap<String, Property>, io::Error> {
        let mut out_properties: HashMap<String, Property> = Default::default();
        let yaml = YamlLoader::load_from_str(property_yaml);
        match yaml {
            Ok(y) => {
                if let Yaml::Hash(h) = &y[0] {
                    for (key, value) in h.iter() {
                        let new_prop: Property = Self::parse_yaml_property(value);
                        if let Yaml::String(k) = key {
                            out_properties.insert(k.to_string(), new_prop);
                        }  
                    }
                }
            },
            Err(e) => {
                return Err(io::Error::new(io::ErrorKind::Other, format!("Error parsing yaml! {:?}", e)));
            }
        }

        Ok(out_properties)
    }

    fn parse_yaml_property(in_prop: &Yaml) -> Property {
        match in_prop {
            Yaml::Real(p) => return Property::Number(p.parse::<f64>().expect("FAILED TO PARSE FLOAT")),
            Yaml::Integer(p) => return Property::Number((*p) as f64),
            Yaml::String(p) => return Property::Text(p.clone()),
            Yaml::Boolean(p) => return Property::Checkbox(*p),
            Yaml::Array(p) => {
                let mut out_arr: Vec<Property> = vec![];
                for i in p {
                    out_arr.push(Self::parse_yaml_property(i));
                }
                return Property::List(out_arr)
            },
            _ => return Property::Unknown,
        }
    }
}
