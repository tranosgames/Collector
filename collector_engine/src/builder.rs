use crate::collect::Collect;
use crate::writer::Writer;
use crate::parser::{YamlParser,YamlFile};
// use log::*;

pub struct Collector {
	pub source: String,
	pub destinateion: String,
	pub ressource_path: String,
	pub artefact_selection: Vec<String>,
	writer: Option<Writer>,
	collect: Option<Collect>,
	parser: Option<YamlParser>
}

impl Collector {
	pub fn new(source: &str, destinateion: &str, ressource_path: &str, artefact_selection: Vec<String>) -> Self{
		Collector {
			source: source.to_string(),
			destinateion: destinateion.to_string(),
			ressource_path: ressource_path.to_string(),
			artefact_selection: artefact_selection,
			writer: None,
			collect: None,
			parser: None,
		}
	}

	pub async fn start(&self){
		let parser_obj: YamlParser = self.parser.unwrap();
		if self.parser.is_none(){
			let parser_obj: YamlParser = YamlParser::new(self.ressource_path, self.artefact_selection);
		}
		let listor = parser_obj.get_yaml_file();
    	let doc_artefact: Vec<YamlFile> = parser_obj.get_doc_struct(listor).await;
    	parser_obj.select_artefact(&self.list_artefact_to_use,doc_artefact);
	}


	fn import_writer(&mut self, writer: Writer){
		self.writer = Some(writer);
	}

	fn import_collect(&mut self, collect: Collect){
		self.collect = Some(collect);
	}

	fn import_parser(&mut self, parser: YamlParser){
		self.parser = Some(parser);
	}

}