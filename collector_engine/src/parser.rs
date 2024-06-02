use std::path::PathBuf;
use tokio::fs;
use glob::glob;
use serde::{Serialize,Deserialize};

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct YamlArtifact {
	pub metadata: Metadata,
	pub artifact: Artifact
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
pub struct Artifact{
	pub path: Option<Vec<String>>,
	pub group: Option<Vec<String>>
}

#[derive(Clone)]
pub struct YamlParser {
	pub ressource_path: String,
	artifact_element_glob: Vec<String>,
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
			artifact_element_glob: Vec::new(),
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

	pub async fn get_doc_struct(&self, list_file: Vec<PathBuf>) -> Vec<YamlArtifact> {
		let mut parse_file = Vec::new();
		for file in list_file{
			let reader = fs::read_to_string(file.clone()).await;
			for document in serde_yml::Deserializer::from_str(&reader.unwrap()){
				let value = YamlArtifact::deserialize(document);
				match &value.as_ref().unwrap().artifact.path{
					None => {
						match &value.as_ref().unwrap().artifact.group{
							None => panic!("Error of file {:?}: artifact.group and artifact.path have not been found!", &file),
							Some(_) => ()
						}
					}
					Some(_) => {
						match &value.as_ref().unwrap().artifact.group{
							None => (),
							Some(_) => panic!("Error of file {:?}: artifact.group and artifact.path have been found, please select a choice element!", &file)
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

	pub fn select_artifact(&mut self, artifacts_name: Vec<String>, doc_artifact: Vec<YamlArtifact>) -> Vec<String>{
		let get_doc_artifact = doc_artifact;
		for artifact_want in artifacts_name{
			let get = &get_doc_artifact.iter().find(|e| e.metadata.name == artifact_want);
			match get {
				Some(struct_element) => {
					match &struct_element.artifact.group {
						Some(name_artifact_file) => self.select_artifact(name_artifact_file.to_vec(),get_doc_artifact.clone()),
						None => Vec::new()
					};
					match &struct_element.artifact.path {
						Some(name_artifact_elements) => name_artifact_elements.iter().for_each(|e| {
								if !&self.artifact_element_glob.contains(e){
									&self.artifact_element_glob.push(e.to_string())
								}else{&()};
							}
						),
						None => ()
					};
				},
				None => panic!("Error of artifact argument : \"{}\" name not found in file ressources",&artifact_want),
			}
		}
		self.artifact_element_glob.clone()
	}
}