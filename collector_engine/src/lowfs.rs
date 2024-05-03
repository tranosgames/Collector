use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use anyhow::{Result,anyhow};

use std::io::{Read,Seek,BufReader};
use std::path::{PathBuf,Path};

use ntfs::{Ntfs,NtfsFile,NtfsReadSeek};
use ntfs::indexes::NtfsFileNameIndex;

use crate::sector_reader::SectorReader;

struct InfoFS<'n, T>
where
    T: Read + Seek,{
	current_directory: Vec<NtfsFile<'n>>,
	fs: T,
    ntfs: &'n Ntfs,
}

pub async fn extract_ntfs(drive_letter: &str, artefact_name: &str, output_file: File) -> Result<String,>{
	let mut concat_source = String::from("\\\\?\\");
	let mut drive_letter = drive_letter.to_string() ;
	if drive_letter.ends_with("\\"){
		let _ = &drive_letter.pop();
	}
	let _ = &concat_source.push_str(&drive_letter);
	let src_path: &Path = &PathBuf::from(&concat_source);
	// Create ntfs struct
	let f = std::fs::File::open(src_path)?;
	let sr = SectorReader::new(f,4096)?;
	let mut fs = BufReader::new(sr);	
	let mut ntfs = Ntfs::new(&mut fs)?;
	ntfs.read_upcase_table(&mut fs)?;
	let current_directory = vec![ntfs.root_directory(&mut fs)?];
	let mut infofs = InfoFS{
		current_directory,
		fs,
		ntfs: &ntfs
	};
	
	// Find file or path
	let mut split_artefact: Vec<&str> = artefact_name.split('\\').collect();
	let binding = split_artefact.clone();
 	let get_filename = binding.last();
	let _ = &split_artefact.pop();
	if split_artefact.len() > 1 {
		let _ = move_to(&mut infofs, split_artefact);
		let file = from_to(&mut infofs, get_filename.unwrap());
		match file{
			Ok(_) => {
				// let output_file = writer::create_output_file(dest_file,artefact_name).await;	
				let _ = write_out(&mut infofs,output_file,file?).await;
			}
			Err(e) => return Err(anyhow!(e)), 
		}
	}else{
		let file = from_to(&mut infofs, artefact_name);
		match file{
			Ok(_) => {
				// let output_file = writer::create_output_file(dest_file,artefact_name).await;	
				let _ = write_out(&mut infofs,output_file,file?).await;
			}
			Err(e) => return Err(anyhow!(e)), 
		}
	}
	let _ = &concat_source.push_str("\\");
	let _full_artefact_path = concat_source.push_str(&artefact_name);
	Ok(format!("A file has been recover: {}",&concat_source))
}

fn move_to<T>(infofs: &mut InfoFS<T>,artefact_path: Vec<&str>) -> Result<()>
where
    T: Read + Seek,
{
	for path in artefact_path{
		let index = infofs.current_directory.last().unwrap().directory_index(&mut infofs.fs)?;
		let mut finder = index.finder();
		let entry = NtfsFileNameIndex::find(&mut finder, infofs.ntfs, &mut infofs.fs, path);
		if entry.is_none(){
			return Ok(())
		}
		let file = entry.unwrap()?.to_file(infofs.ntfs, &mut infofs.fs)?;
		infofs.current_directory.push(file);
	}
	Ok(())

}

fn from_to<'n,T>(infofs: &mut InfoFS<'n, T>,artefact_filename: &str) -> Result<NtfsFile<'n>,>
where
	T: Read + Seek,
{
	let index = infofs.current_directory.last().unwrap().directory_index(&mut infofs.fs)?;
	let mut finder = index.finder();
	let entry = NtfsFileNameIndex::find(&mut finder, infofs.ntfs, &mut infofs.fs, artefact_filename);
	let test_entry = match entry {
		Some(entry) => entry,
		None => return Err(anyhow!("Error on {}:",artefact_filename)),
	};
	let file = test_entry.unwrap().to_file(infofs.ntfs, &mut infofs.fs)?;
	Ok(file)
}

async fn write_out<T>(infofs: &mut InfoFS<'_, T>,mut output_file: File, file: NtfsFile<'_>) -> Result<()>
where
	T: Read + Seek,
{
	let data_item = file.data(&mut infofs.fs, "");
	let data_item = data_item.unwrap()?;
	let data_attribute = data_item.to_attribute()?;
	let mut data_value = data_attribute.value(&mut infofs.fs)?;
	let mut buf = [0u8; 32768];
	loop {
		let bytes_read = data_value.read(&mut infofs.fs, &mut buf)?;
		if bytes_read == 0 {
			break;
		}

		output_file.write_all(&buf[..bytes_read]).await?;
	}
	Ok(())
}