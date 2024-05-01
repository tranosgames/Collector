use tokio::fs::DirEntry;
use std::ffi::OsString;
use collector_vss::Vss;

// use log::*;
// use anyhow::{Result,anyhow};
use std::path::PathBuf;
use std::env;
use tokio::fs;
use uuid::Uuid;

pub struct CollectVss {
	pub drive_letter: String,
	pub dst: String,
	pub artefacts_glob: Vec<String>,
}

impl CollectVss {
	pub fn new(drive_letter:&str, dst: &str, artefacts_glob: Vec<String>) -> Self{
		CollectVss {
			drive_letter: drive_letter.to_string(),
			dst: dst.to_string(),
			artefacts_glob: artefacts_glob,
		}
	}

	pub fn get_list(){
		let _vss_engine = Vss::new("C:\\".to_string());
		// println!("{:?}",vss_engine.get_all_list());
		// vss_engine.mount_vss(&PathBuf::from("symlinkvss"));
	}

	pub async fn collect(&self){
		let output_temp_path = self.mount().await;
		println!("{:?}",output_temp_path);
		let mut entry = fs::read_dir(output_temp_path.clone()).await.unwrap();
		loop {
			let next_entry = entry.next_entry().await.unwrap();
			if next_entry.is_none(){
				break
			}
			let get_entry: DirEntry =  next_entry.unwrap();
			let get_file_type = get_entry.file_type().await.unwrap().clone();
			if get_file_type.is_symlink(){
				let name_dir: OsString = get_entry.file_name();
				let dir_as_path: PathBuf = get_entry.path();
				println!("{:?}",name_dir);
				println!("{:?}",dir_as_path);
			}
		}


		let _end_vss = fs::remove_dir_all(&output_temp_path).await;		
	}

	async fn mount(&self) -> PathBuf{
		let mk_temp_d: PathBuf = env::temp_dir();
		let temp_vss_dir: PathBuf = mk_temp_d.join(Uuid::new_v4().to_string());
		let _ = fs::create_dir(&temp_vss_dir).await;
		let dl = &self.drive_letter;
		let vss_obj = Vss::new(dl.to_string());
		let _mount_vss = vss_obj.mount_vss(&temp_vss_dir);
		temp_vss_dir
	}
}