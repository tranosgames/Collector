use std::io::{Write,Read};
use std::path::{Path, PathBuf};
use walkdir::{WalkDir,DirEntry};
use zip::{write::FileOptions,ZipWriter};
use tokio::fs;
use tokio::fs::File;
use anyhow::Result;


pub struct Writer {
	pub dst: PathBuf,
}


impl Writer {
	pub fn new(dst: &str) -> Self{
		let entry_to_path: PathBuf = PathBuf::from(dst);
		Writer {
			dst: entry_to_path,
		}
	}

	pub fn get_filepath(&self, path_name: &str) -> PathBuf{
		let mut format_file: String = path_name.to_string().replace(":","");
		if format_file.starts_with("\\"){
			format_file.remove(0);
		}
		let create_pathbuf: PathBuf = self.dst.join(format_file);
		create_pathbuf
	}

	/// Create a file in output directory.
	/// If string path as given, the entire path will be create.
	/// Output the file descriptor.
	pub async fn create_file(&self, path_name: &str) -> File {
		let get_filepath = self.get_filepath(path_name);
		let _create_folder = self.create_folderpath(path_name).await;
		File::create(get_filepath).await.expect("Impossible to create output file")
	}

	/// create the entire path as given. 
	pub async fn create_folderpath(&self, path_name: &str){
		let mut get_filepath: PathBuf = self.get_filepath(path_name);
		let _pop_filename = get_filepath.pop();
		if !get_filepath.exists(){
			let _ = fs::create_dir_all(get_filepath).await;
		}
	}
	

	/// Zip the destination file by the given name.
	pub fn zip(&self, zip_name: String) -> Result<()>{
		let get_ext = {
			let split_pos = zip_name.char_indices().nth_back(3).unwrap().0;
			&zip_name[split_pos..]
		};
		let mut zip_ext = zip_name.clone();
		if get_ext != ".zip"{
			zip_ext.push_str(".zip");
		}

		// Create zip file
		let path_zip = Path::new(&zip_ext);
		let path_out = Path::new(&self.dst).join(path_zip);

		let file = std::fs::File::create(path_out).unwrap();
		let mut zip = ZipWriter::new(file);
		let options = FileOptions::default()
			.compression_method(zip::CompressionMethod::Bzip2)
			.unix_permissions(0o755);


		
		// Walk in output directory
		let mut buffer = Vec::new();
		let walker = WalkDir::new(&self.dst).into_iter();
		for entry in walker.filter_entry(|e| !self.filter_entry_zip(e,path_zip.to_str().unwrap().to_string())) {
			let unwraper = entry.unwrap(); // DirEntry(".\\out\\Windows")
			let path = unwraper.path(); // ".\\out\\Windows"
			let name = path.strip_prefix(self.dst.clone()).unwrap();
			if path.is_file() {
				#[allow(deprecated)]
				zip.start_file_from_path(name, options)?;
				let mut f = std::fs::File::open(path)?;
				f.read_to_end(&mut buffer)?;
				zip.write_all(&buffer)?;
				buffer.clear();
			}else if !name.as_os_str().is_empty() {
				#[allow(deprecated)]
				zip.add_directory_from_path(name, options)?;

			}
		}
		zip.finish()?;
		Ok(())
	}
	
	fn filter_entry_zip(&self,entry: &DirEntry, name_file: String) -> bool {
		entry.file_name()
			 .to_str()
			 .map(|s| s == name_file)
			 .unwrap_or(false)
	}
}


impl Clone for Writer{
	fn clone(&self) -> Self {
		Writer{ dst: self.dst.clone() }
	}
}

