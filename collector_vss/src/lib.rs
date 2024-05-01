pub mod info;

use crate::info::{VSSObj,DriveLetter};
use std::os::windows;
use std::path::PathBuf;


#[derive(Debug)]
pub struct Vss {
    pub drive_letter: String, 
}

impl Vss {
    pub fn new(drive_letter: String) -> Self{
        Vss {
            drive_letter: drive_letter,
        }
    }

    pub fn mount_vss(&self,dest_path: &PathBuf){
        let vss_list = self.get_all_list();
        for vss_item in vss_list{
            let get_vss_name = vss_item.device_volume_name.split("\\").last().unwrap();
            let concat_dst_name = dest_path.join(get_vss_name);
            let _ = windows::fs::symlink_dir(vss_item.original_volume_name, concat_dst_name);
        }
    }

    pub fn get_all_list(&self) -> Vec<VSSObj>{
        VSSObj::get_list().unwrap()
    }

    pub  fn get_list(&self){

    }

    pub fn convert(&self){
        let dl = &self.drive_letter;
        let _dlv = DriveLetter::from(dl.to_string()).to_volume();
    }


}

#[cfg(test)]
mod tests {

    #[test]
    fn extract_vss_list() {
    }
}
