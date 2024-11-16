use std::io;
use std::fs::{self, File, DirEntry};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use chrono::{ DateTime, Local };
use indexmap::{ IndexMap };
use regex::Regex;
use yaml_rust::{ YamlLoader, Yaml };

#[derive(Debug)]
pub enum Property {
    Text(String),
    Number(f64),
    Checkbox(bool),
    List(Vec<Property>),
    Date(DateTime<Local>),
    Unknown,
}

#[derive(Debug)]
enum FileFolder {
    File(FileItem),
    Folder(FolderItem),
}

#[derive(Debug)]
pub struct FileItem {
    name: String,
    file_type: String,
    path: PathBuf,
    local_path: PathBuf,
    properties: HashMap<String, Property>,
    tags: Vec<String>,
}

#[derive(Debug)]
pub struct FolderItem {
    name: String,
    path: PathBuf,
    local_path: PathBuf,
}

#[derive(Debug)]
pub struct VaultIndex {
    path: Option<PathBuf>,
    files: IndexMap<String, FileItem>,
    folders: Vec<FolderItem>,
    filepath_ref: IndexMap<String, String>,
}

impl VaultIndex {
    pub fn new(path_to_vault: Option<&str>) -> Result<Self, io::Error> {
        let mut path: PathBuf = PathBuf::new();
        let mut files: IndexMap<String, FileItem> = IndexMap::new();
        let mut folders: Vec<FolderItem> = vec![];
        let mut filepath_ref: IndexMap<String, String> = IndexMap::new();
        if let Some(vault_path) = path_to_vault {
            let p = PathBuf::from(vault_path);
            path = p.clone();
            if !p.is_dir() {
                return Err(io::Error::new(io::ErrorKind::NotFound, "The path specified either could not be found, could not be accessed, or was not a directory."));
            }

            let file_collection = Self::recursive_generate_filefolders(&p, &p);
            
            for file in file_collection {
                match file {
                    FileFolder::File(fi) => {
                        filepath_ref.insert(fi.name.clone(), fi.local_path.to_str().unwrap().to_string().clone());
                        files.insert(fi.name.clone(), fi);
                    },
                    FileFolder::Folder(fi) => {
                        folders.push(fi);
                    },
                }
            }

            println!("{:?}", files);
            println!("{:?}", filepath_ref);
        }

        let vi = VaultIndex {
            path: Some(path),
            files,
            folders,
            filepath_ref,
        };

        Ok(vi)
    }

    fn recursive_generate_filefolders(dir_path: &PathBuf, vault_path: &PathBuf) -> Vec<FileFolder> {
        let mut out_filefolders: Vec<FileFolder> = vec![];
        let fpath = fs::read_dir(dir_path);
        match fpath {
            Ok(paths) => {
                for path in paths {
                    let child_file = path.unwrap();
                    if child_file.file_type().unwrap().is_dir() {
                        out_filefolders.push(Self::generate_folder_item(&child_file.path(), vault_path).unwrap());
                        let mut children_filepaths = Self::recursive_generate_filefolders(&child_file.path(), vault_path);
                        out_filefolders.append(&mut children_filepaths);
                    } else {
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
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let file_type = path.extension().unwrap().to_str().unwrap().to_owned();
        let local_path = path.strip_prefix(vault_path).unwrap().to_path_buf();
        // Figure out a way to remove these from the loop
        let tag_matcher = Regex::new(r"(\B#\S+)").expect("REGEX FAILED");
        let properties_matcher = Regex::new(r"(---[\w\W]*?---)").expect("REGEX FAILED");

        let mut tags: Vec<String> = vec![];
        let mut properties: HashMap<String, Property> = Default::default();
        
        let file_contents = fs::read_to_string(path);

        match file_contents {
            Ok(cont) => {
                for (_, [tag]) in tag_matcher.captures_iter(&cont).map(|c| c.extract()) {
                    tags.push(tag.to_string().replace('#', ""));
                }

                if let Some(ind) = cont.find("---") {
                    if ind == 0 {
                        let properties_match = properties_matcher.captures(&cont).unwrap();
                        properties = Self::generate_properties(properties_match.get(0).unwrap().as_str().replace("---", "").trim()).unwrap();

                    }
                }
            },
            Err(e) => {
                return Err(e);
            }
        }

        let fi = FileItem {
            name,
            file_type,
            path: path.to_path_buf(),
            local_path,
            properties,
            tags,
       };
       Ok(FileFolder::File(fi))
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
                return Err(io::Error::new(io::ErrorKind::Other, "Error parsing yaml!"));
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
