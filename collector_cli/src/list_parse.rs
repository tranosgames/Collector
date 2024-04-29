use collector_engine::parser::YamlArtefact;

pub struct ArtefactListing {
	pub artefact_struct_list: Vec<YamlArtefact>,
}

#[derive(Debug)]
pub struct ArtefactCategoryNames {
	category_name: String,
	list_name_artefact: Vec<String>,
}

impl ArtefactListing {
	pub fn load(artefact_struct: Vec<YamlArtefact>) -> Self {
		ArtefactListing {
			artefact_struct_list: artefact_struct,
		}
	}

	pub fn names_pa(&self) -> Vec<String>{
		let mut names: Vec<String> = Vec::new();
		for artefact in &self.artefact_struct_list{
			if artefact.artefact.path.is_some(){
				names.push(artefact.metadata.name.clone());
			}
		}
		names
	}

	pub fn names_gr(&self) -> Vec<String>{
		let mut names: Vec<String> = Vec::new();
		for artefact in &self.artefact_struct_list{
			if artefact.artefact.group.is_some(){
				names.push(artefact.metadata.name.clone());
			}
		}
		names
	}

	pub fn list_categories(&self) -> Vec<ArtefactCategoryNames>{
		let mut names: Vec<ArtefactCategoryNames> = Vec::new();
		for artefact in &self.artefact_struct_list{
			if artefact.metadata.category.is_some(){
				let category = artefact.metadata.category.clone().unwrap();
				let finder = names.iter_mut().find(|e| e.category_name == category);
				if finder.is_some(){
					let choosen: &mut ArtefactCategoryNames = finder.unwrap();
					let _ = &choosen.list_name_artefact.push(artefact.metadata.name.clone());
				}else{
					let build = ArtefactCategoryNames{
						category_name: category,
						list_name_artefact: vec![artefact.metadata.name.clone()],
					};
					names.push(build);
				}
			}else{
				let finder = names.iter_mut().find(|e| e.category_name == "Other".to_string());
				if finder.is_some(){
					let choosen: &mut ArtefactCategoryNames = finder.unwrap();
					let _ = &choosen.list_name_artefact.push(artefact.metadata.name.clone());
				}else{
					let build = ArtefactCategoryNames{
						category_name: "Other".to_string(),
						list_name_artefact: vec![artefact.metadata.name.clone()],
					};
					names.push(build);
				}
			}
		}
		names
	}
}