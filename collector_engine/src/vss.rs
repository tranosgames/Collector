use std::ffi::OsStr;
// use std::path::Path;
use crate::collect::Collect;
use collector_vss::{Vss,info::VSSObj};

// use tokio::fs::DirEntry;
// use std::ffi::OsString;

use log::*;
// use anyhow::{Result,anyhow};
use std::path::PathBuf;
use std::env;
use tokio::fs;
use uuid::Uuid;

pub struct CollectVss {
	pub drive_letter: String,
	pub dst: String,
	pub list_artefacts: Vec<String>,
	vss_obj: Vss
}

impl CollectVss {
	pub fn new(drive_letter:&str, dst: &str, list_artefacts: Vec<String>) -> Self{

		CollectVss {
			drive_letter: drive_letter.to_string(),
			dst: dst.to_string(),
			list_artefacts: list_artefacts,
			vss_obj: Vss::new(drive_letter.to_string()),
		}
	}

	pub fn get_list(&self){
		let vss_engine = Vss::new("C:\\".to_string());
		let _m = vss_engine.get_list();
		// println!("{:?}",m);
		// println!("{:?}",vss_engine.get_all_list());
		// vss_engine.mount_vss(&PathBuf::from("symlinkvss"));
	}

	pub async fn collect(&self){
		let vss_list_item: Vec<VSSObj> = match self.vss_obj.get_list() {
			Ok(is_list) =>   is_list,
			Err(_get_err) => {println!("{:?}",_get_err );return},
		};

		// Create temporary path to store vss
		let mk_temp_d: PathBuf = env::temp_dir();
		let temp_vss_dir: PathBuf = mk_temp_d.join(Uuid::new_v4().to_string());
		let _ = fs::create_dir(&temp_vss_dir).await;

		for vss_item in vss_list_item{
			let mounted_vss: PathBuf = Vss::mount_vss(vss_item.clone(), temp_vss_dir.clone()).await;

			if mounted_vss.is_symlink(){
				let dir_as_path: PathBuf = mounted_vss.clone();
				let vss_path_str: &str = dir_as_path.to_str().unwrap();
				let get_dst = &self.dst;
				let dst_as_path: PathBuf = PathBuf::from(get_dst);
				let name_dir: &OsStr = mounted_vss.file_name().unwrap();
				let concat_dst_vss: PathBuf = dst_as_path.join(name_dir);
				let end_dst_str: &str = concat_dst_vss.to_str().unwrap();
				info!("[VSS] Start collecting VSS");
				let mut collector_obj = Collect::new(vss_path_str,end_dst_str,self.list_artefacts.clone(),true);
				collector_obj.vss(vss_item.clone());
	  		  	collector_obj.start().await;
				info!("[VSS] End collecting VSS");
			}
		
		}
		let _end_vss = fs::remove_dir_all(&temp_vss_dir).await;		
	}
}