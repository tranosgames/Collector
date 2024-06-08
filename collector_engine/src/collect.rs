use crate::writer::Writer;
use crate::helper::{FormatSource,is_admin};
use crate::extract::{try_filesystem,try_ntfs};
use crate::csv::{CsvLog,CsvLogItem};
use crate::mount::info::VSSObj;


use tokio::fs::File;
use tokio::io::AsyncReadExt;
use filetime::FileTime;
use std::path::PathBuf;
use glob::glob;
use anyhow::Result;
use sha1::{Sha1,Digest};
// use log::*;


pub struct Collect {
	pub src: String,
	pub dst: String,
	pub artifacts_glob: Vec<String>,
	writer: Writer,
	vss_item: Option<VSSObj>,
	csv_copy: CsvLog,
}

impl Collect{
	pub async fn new(src: String, dst: String, artifacts_glob: Vec<String>) -> Collect{
		let create_writer: Writer = Writer::new(dst.clone());
		let csv_filename = create_writer.get_filepath_as_str("Collector_copy.csv".into());
		let _create_csv = create_writer.create_file("Collector_copy.csv".into()).await;
		Collect { 
			src: src.to_string(), 
			dst: dst.to_string(), 
			artifacts_glob: artifacts_glob,
			writer: create_writer,
			vss_item: None,
			csv_copy: CsvLog::new(csv_filename).await,
		}
	}

	pub async fn start(&mut self){
		if !is_admin(){
			panic!("You need to run as Administrator!");
		}
		let artifact_iter = self.artifacts_glob.clone();
		for artifact in artifact_iter {
			let mut artifact_element = artifact.to_string();
			if artifact_element.starts_with("\\"){
				artifact_element.remove(0);
			}

			let src_path: PathBuf = FormatSource::from(&self.src).to_path();
			let src_path_artifact =  src_path.join(artifact_element);

			let source_with_artifact_out: &str = src_path_artifact.to_str().unwrap();
			for entry in glob(source_with_artifact_out).expect("Error for parsing artifact"){
				let mut to_entry = entry.unwrap(); // DirEntry()
				if to_entry.as_path().is_file(){
					let for_metadata = to_entry.clone();
					let mut mod_entry: String = to_entry.to_str().unwrap().to_string();
					if self.vss_item.is_some(){
						let get_vss_item = self.vss_item.clone().unwrap();
						let vss_as_path = PathBuf::from(&get_vss_item.device_volume_name);
						let get_vss_name = vss_as_path.file_name().unwrap();
						mod_entry = mod_entry.replace(&self.src, get_vss_name.to_str().unwrap());
					}

					let mut output_file: File = self.writer.create_file(mod_entry.clone()).await;

					// For filesystem
					match try_filesystem(to_entry.clone(),&mut output_file).await {
						Ok(_) => {
							let filepath_art = self.writer.get_filepath_as_str(mod_entry.clone()).clone();
							self.to_csv(mod_entry.clone(),filepath_art,false).await;
							continue;
						},
						Err(_) => ()
					}

					// For ntfs
					let item: Option<VSSObj> = self.vss_item.clone();
					if item.is_some(){
						let pathbuf_to_str = to_entry
							.to_str()
							.unwrap()
							.to_string();
						let add_backslah = self.src.clone() + "\\";
						to_entry = pathbuf_to_str.replace(&add_backslah,"").into();
					}
					match try_ntfs(to_entry,&mut output_file,item).await {
						Ok(_) => {
							// Set Metadata on new file
							let metadata = std::fs::metadata(for_metadata).expect("Failed to extract metedata");
							let mtime = FileTime::from_last_modification_time(&metadata);
							let atime = FileTime::from_last_access_time(&metadata);
							let resolver = self.writer.get_filepath(mod_entry.clone());
							let _ = filetime::set_file_times(resolver,atime,mtime);

							let filepath_art = self.writer.get_filepath_as_str(mod_entry.clone());
							self.to_csv(mod_entry.clone(),filepath_art,true).await;
						},
						Err(_) => ()
					}
				}
			}
		}
	}

	pub async fn zip(&self,zip_password: Option<String>) -> Result<()>{
		let zipping = self.writer.zip(zip_password);
		zipping.await
	}

	pub fn vss(&mut self, vss_item: VSSObj){
		self.vss_item = Some(vss_item);
	}

	async fn to_csv(&mut self, source_artifact: String, destination_artifact: String , from_ntfs: bool){
		let mut log_item: CsvLogItem = Default::default();
		log_item.source_file = source_artifact.clone();
		log_item.destination_file = destination_artifact.clone();

		let metadata = std::fs::metadata(&destination_artifact).expect("Failed to extract metedata");
		let mtime = FileTime::from_last_modification_time(&metadata);
		let atime = FileTime::from_last_access_time(&metadata);

		log_item.modfified_time = mtime.to_string();
		log_item.access_time = atime.to_string();
		log_item.from_ntfs = from_ntfs;


		let mut get_file = File::open(destination_artifact).await.unwrap();
		let mut hasher = Sha1::new();
		let mut contents = [0u8; 4092];
		loop {
			let reader = get_file.read(&mut contents).await;
			if reader.unwrap() == 0{
				break;
			}
			hasher.update(contents);
		}
		log_item.hasfile_sha256 = hex::encode(hasher.finalize());

		let _ = self.csv_copy.add_row_struct(log_item).await;
	}

}