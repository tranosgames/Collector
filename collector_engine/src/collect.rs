use tokio::fs::File;
use filetime::FileTime;
use crate::lowfs;
use crate::writer::Writer;
use collector_vss::info::VSSObj;

use std::path::{PathBuf};
use regex::Regex;
use glob::glob;
use tokio::fs;
use log::*;
use anyhow::Result;


#[derive(Clone)]
pub struct Collect {
	pub src: String,
	pub dst: String,
	pub artefacts_glob: Vec<String>,
	writer: Writer,
	vss_item: Option<VSSObj>,
	remove_src_file: bool,
}

impl Collect{
	pub fn new(src: &str, dst: &str, artefacts_glob: Vec<String>, remove_src_file: bool) -> Collect{
		let create_writer: Writer = Writer::new(dst);
		Collect { 
			src: src.to_string(), 
			dst: dst.to_string(), 
			artefacts_glob: artefacts_glob,
			writer: create_writer,
			vss_item: None,
			remove_src_file: remove_src_file,
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
			for entry in glob(source_with_artefact_out).expect("Error for parsing artefact"){
				let to_entry = entry.unwrap();
				if to_entry.as_path().is_file(){
					let mut mod_entry: String = to_entry.to_str().unwrap().to_string().replace(":","");
					if self.remove_src_file{
						let check_src = self.src.replace(":","");
						let mod_dst_str: String  = mod_entry.replace(&check_src,"");
						mod_entry = mod_dst_str;
					}

					let output_file: File = self.writer.create_file(&mod_entry).await;
					let high_level_res = self.try_high_level(&to_entry).await;
					match high_level_res {
						Ok(_) => continue,
						Err(_) => ()
					}
					match self.try_low_level(&to_entry,output_file).await {
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

	pub fn vss(&mut self, vss_item: VSSObj){
		self.vss_item = Some(vss_item);
	}

	async fn try_high_level(&self, artefact_entry: &PathBuf) -> Result<(),>{
		let format_entry: &str = artefact_entry.to_str().unwrap();
		let mut get_path_file: PathBuf = self.writer.get_filepath(format_entry);
		if self.remove_src_file{
			let check_src = self.src.replace(":","");
			let mod_dst = get_path_file.to_str().unwrap().to_string();
			let mod_dst_str = mod_dst.replace(&check_src,"");
			get_path_file = PathBuf::from(mod_dst_str);
		}
		if let Err(err) = fs::copy(format_entry, get_path_file).await {
			error!("Impossible to extract file in user land: {}",format_entry);
			Err(err.into())
		}else{
			info!("A file has been recover from user land: {}",format_entry);
			Ok(())
		}
	}

	async fn try_low_level(&self, artefact_entry: &PathBuf, filer: File) -> Result<(),>{
		let format_entry: &str = artefact_entry.to_str().unwrap();
		let drive_letter: String = self.get_drive_letter(format_entry).expect("Can't get drive letter from this path");
		let mut volume_entry: String = drive_letter.clone(); 
		if self.vss_item.is_some(){
			let vss_volume = self.vss_item.clone().unwrap();
			volume_entry = vss_volume.device_volume_name.replace("\\\\?\\","");
		}
		
	 	// Create output file 
	 	let mut output_path: String = format_entry.to_string();
	 	if self.remove_src_file{
			let mod_dst_str: String  = output_path.replace(&self.src,"");
			output_path = mod_dst_str;
		}
		let available_artefact = output_path.replace(&drive_letter,"");
		let out_info = lowfs::extract_ntfs(&volume_entry,&available_artefact, filer).await;
		
		// Set Metadata on new file
		let metadata = std::fs::metadata(format_entry).expect("Failed to extract metedata");
		let mtime = FileTime::from_last_modification_time(&metadata);
		let atime = FileTime::from_last_access_time(&metadata);
		let resolver = self.writer.get_filepath(&output_path);
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
		let drive_letter_regex = Regex::new(r"(^[A-Za-z]:\\)").expect("Failed to parse regex");
		let caps = drive_letter_regex.captures(format_path);
		if caps.is_some(){
			let drive_letter = caps.unwrap().get(0).map_or("", |m| m.as_str());
			Some(drive_letter.to_string())      
		}else{
			None
		}
	}

}