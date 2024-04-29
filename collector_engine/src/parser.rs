use std::path::PathBuf;
use tokio::fs;
use glob::glob;
use serde::{Serialize,Deserialize};

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct YamlArtefact {
	pub metadata: Metadata,
	pub artefact: Artefact
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Metadata {
	pub name: String,
	pub description: String,
	pub date: Option<String>,
	pub category: Option<String>,
	pub source: Option<Vec<String>>,
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Artefact{
	pub path: Option<Vec<String>>,
	pub group: Option<Vec<String>>
}

#[derive(Clone)]
pub struct YamlParser {
	pub ressource_path: String,
	artefact_element_glob: Vec<String>,
}

impl YamlParser{
	pub fn new(ressource_path: String) -> Self{
		let mut format_ressource_path = ressource_path;
    	if !format_ressource_path.ends_with('\\'){
    	    let _ = format_ressource_path.push('\\');
    	}
		let _format_ressource_path = format_ressource_path.push_str("**/*.yaml");
		YamlParser {
			ressource_path: format_ressource_path.to_string(),
			artefact_element_glob: Vec::new(),
		}
	}

	pub fn get_yaml_file(&self) -> Vec<PathBuf> {
		let mut list_yaml_file = Vec::new();
		for entry in glob(&self.ressource_path).expect("Failed to read glob pattern") {
			let path_to_string = entry.unwrap();
			list_yaml_file.push(path_to_string.to_path_buf());
		}
		list_yaml_file
	}

	pub async fn get_doc_struct(&self, list_file: Vec<PathBuf>) -> Vec<YamlArtefact> {
		let mut parse_file = Vec::new();
		for file in list_file{
			let reader = fs::read_to_string(file.clone()).await;
			for document in serde_yml::Deserializer::from_str(&reader.unwrap()){
				let value = YamlArtefact::deserialize(document);
				match &value.as_ref().unwrap().artefact.path{
					None => {
						match &value.as_ref().unwrap().artefact.group{
							None => panic!("Error of file {:?}: artefact.group and artefact.path have not been found!", &file),
							Some(_) => ()
						}
					}
					Some(_) => {
						match &value.as_ref().unwrap().artefact.group{
							None => (),
							Some(_) => panic!("Error of file {:?}: artefact.group and artefact.path have been found, please select a choice element!", &file)
						}
					}
				}
				let out = match value {
					Ok(expr) => expr,
					Err(e) => panic!("Error of file {:?}: {:?}", &file,e.to_string()),
				};
				parse_file.push(out);
			}
		}
		parse_file
	}

	pub fn select_artefact(&mut self, artefacts_name: Vec<String>, doc_artefact: Vec<YamlArtefact>) -> Vec<String>{
		let get_doc_artefact = doc_artefact;
		for artefact_want in artefacts_name{
			let get = &get_doc_artefact.iter().find(|e| e.metadata.name == artefact_want);
			match get {
				Some(struct_element) => {
					match &struct_element.artefact.group {
						Some(name_artefact_file) => self.select_artefact(name_artefact_file.to_vec(),get_doc_artefact.clone()),
						None => Vec::new()
					};
					match &struct_element.artefact.path {
						Some(name_artefact_elements) => name_artefact_elements.iter().for_each(|e| {
								if !&self.artefact_element_glob.contains(e){
									&self.artefact_element_glob.push(e.to_string())
								}else{&()};
							}
						),
						None => ()
					};
				},
				None => panic!("Error of artefact argument : \"{}\" name not found in file ressources",&artefact_want),
			}
		}
		self.artefact_element_glob.clone()
	}
}