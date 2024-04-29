use filetime::FileTime;
use crate::lowfs;
use crate::writer::Writer;

use std::path::{PathBuf};
use regex::Regex;
use glob::glob;
use tokio::fs;
use log::*;
use anyhow::Result;

pub struct Collect {
	pub src: String,
	pub dst: String,
	pub artefacts_glob: Vec<String>,
	writer: Writer
}

impl Collect{
	pub fn new(src: &str, dst: &str, artefacts_glob: Vec<String>) -> Collect{
		let create_writer: Writer = Writer::new(dst);
		Collect { 
			src: src.to_string(), 
			dst: dst.to_string(), 
			artefacts_glob: artefacts_glob,
			writer: create_writer
		}
	}

	pub async fn start(&mut self){
		let artefact_iter = &self.artefacts_glob;
		if !self.src.ends_with("\\"){
			self.src.push('\\');
		}
		for artefact in artefact_iter {
			let src_path: PathBuf = PathBuf::from(self.src.clone());
			let mut format_artefact_name: String = artefact.to_string();
			if format_artefact_name.starts_with("\\"){
				format_artefact_name.remove(0);
			}
			let src_path_artefact =  src_path.join(format_artefact_name);
			let source_with_artefact_out = src_path_artefact.to_str().unwrap();
		println!("enter");
			for entry in glob(source_with_artefact_out).expect("Error for parsing artefact"){
				let to_entry = entry.unwrap();
				if to_entry.clone().as_path().is_file(){
					self.writer.create_file(to_entry.clone().to_str().unwrap()).await;
					let high_level_res = self.try_high_level(&to_entry).await;
					match high_level_res {
						Ok(_) => continue,
						Err(_) => ()
					}
					match self.try_low_level(&to_entry).await {
						Ok(_) => (),
						Err(_) => (),
					}
				}
			}
		}
	}

	pub fn zip(&self,zip_name: String) -> Result<()>{
		let zipping = self.writer.zip(zip_name);
		zipping
	}

	async fn try_high_level(&self, artefact_entry: &PathBuf) -> Result<(),>{
		let format_entry = artefact_entry.to_str().unwrap();
		let get_path_file = self.writer.get_filepath(format_entry);
		if let Err(err) = fs::copy(format_entry, get_path_file).await {
			error!("Impossible to extract file: {}",format_entry);
			Err(err.into())
		}else{
			info!("A file has been recover: {}",format_entry);
			Ok(())
		}
	}

	async fn try_low_level(&self, artefact_entry: &PathBuf) -> Result<(),>{
		let format_entry: &str = artefact_entry.to_str().unwrap();
		let drive_letter: String = self.get_drive_letter(format_entry).expect("Can't get drive letter from this path");
		let available_artefact = format_entry.replace(&drive_letter,"");
		
	 	// Create output file and set metadata
		let output_file = self.writer.create_file(&available_artefact).await;
		let metadata = std::fs::metadata(format_entry).unwrap();
		let mtime = FileTime::from_last_modification_time(&metadata);
		let atime = FileTime::from_last_access_time(&metadata);
		let resolver = self.writer.get_filepath(&available_artefact);
		let out_info = lowfs::extract_ntfs(&drive_letter,&available_artefact, output_file).await;
		let _ = filetime::set_file_times(resolver,atime,mtime);

		match out_info {
			Ok(res) => {
				info!("{}",res);
				Ok(())
			},
			Err(err) => {
				error!("Impossible to extract file: {} ",format_entry);
				Err(err)
			},
		}
	}

	fn get_drive_letter(&self,path: &str) -> Option<String> {
		let format_path: &str = path;
		let drive_letter_regex = Regex::new(r"(^[A-Za-z]:\\)").unwrap();
		let caps = drive_letter_regex.captures(format_path);
		if caps.is_some(){
			let drive_letter = caps.unwrap().get(0).map_or("", |m| m.as_str());
			Some(drive_letter.to_string())      
		}else{
			None
		}
	}

}

impl Clone for Collect{
	fn clone(&self) -> Self {
		Collect{ src: self.src.clone(), dst: self.dst.clone(), artefacts_glob: self.artefacts_glob.clone() , writer: self.writer.clone() }
	}
}

