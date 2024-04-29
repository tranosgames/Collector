use crate::collect::Collect;

use std::ptr::null_mut;
use std::mem::zeroed;
use log::*;
use anyhow::{Result,anyhow};
use widestring::U16CStr;
use winapi::{
	shared::{
		rpcdce::{
			RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
			RPC_C_IMP_LEVEL_IMPERSONATE
		},
		winerror::{
			S_OK,
			E_ACCESSDENIED,
			E_INVALIDARG
		}
	},
	um::{
		cguid::GUID_NULL,
		combaseapi::{
			CoInitializeEx,
			CoInitializeSecurity,
			COINITBASE_MULTITHREADED
		},
		objidl::EOAC_DYNAMIC_CLOAKING,
		vsbackup::{
			IVssBackupComponents,
			CreateVssBackupComponents,
		},
		vss::{
			IVssEnumObject,
			VSS_BT_FULL,
			VSS_CTX_ALL,
			VSS_OBJECT_PROP,
			VSS_SNAPSHOT_PROP,
			VSS_OBJECT_NONE,
			VSS_OBJECT_SNAPSHOT,
		},
		winnt::HRESULT
	}
};

pub struct Vss {
	pub dst: String,
	pub artefacts_glob: Vec<String>,
}

impl Vss {
	pub fn new(dst: &str, artefacts_glob: Vec<String>) -> Self{
		Vss {
			dst: dst.to_string(),
			artefacts_glob: artefacts_glob,
		}
	}

	pub async fn collect(&self){
		let result_list_vss = match self.get_vss_list() {
			Ok(expr) => expr,
			Err(_err) => {
				error!("Error they are not vss");
				return 
			},
		};
		for vss_path in result_list_vss {
			let vss_without_root_drive = vss_path.replace("\\\\?\\","");
			let mut collector_obj = Collect::new(&vss_without_root_drive,&self.dst,self.artefacts_glob.clone());
		    let _collector_obj_start = collector_obj.start().await;
			println!("{:?}",collector_obj.dst); 
		}
	}

	fn get_vss_list(&self) -> Result<Vec<String>,> {
		let mut vss_list: Vec<String> = Vec::new();
		unsafe {
			let mut backup_components: *mut IVssBackupComponents = null_mut();
			let mut enum_object: *mut IVssEnumObject = null_mut();
			let mut prop: VSS_OBJECT_PROP = zeroed();

			// Initializing COM
			let mut hr: HRESULT = CoInitializeEx(null_mut(), COINITBASE_MULTITHREADED);
			match hr {
				S_OK => {
					info!("[VSS] Initialized COM");
				},
				_ => {
					error!("[VSS] Couldn't Initialize COM");
					return Err(anyhow!("Couldn't Initialize COM"))
				}
			};

			// Initializing COM Security;
			hr = CoInitializeSecurity(
				null_mut(),
				-1,
				null_mut(),
				null_mut(),
				RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
				RPC_C_IMP_LEVEL_IMPERSONATE,
				null_mut(),
				EOAC_DYNAMIC_CLOAKING, 
				null_mut()
			);
			match hr {
				S_OK => {
					info!("[VSS] Initialized COM Security");
				},
				_ => {
					error!("[VSS] Couldn't Initialize COM Security");
					return Err(anyhow!("Couldn't Initialize COM Security"))
				}
			};

			// Creating Backup Components
			hr = CreateVssBackupComponents(&mut backup_components);
			match hr {
				S_OK => {
					info!("[VSS] Created Backup Components");
				},
				E_ACCESSDENIED => {
					error!("[VSS] Run as admin!");
					return Err(anyhow!("Run as admin"))
				}
				_ => {
					error!("[VSS] Couldn't create Backup Components");
					return Err(anyhow!("Couldn't create Backup Components"))
				}
			};


			// Initializing for Backup
			hr = backup_components.as_ref().unwrap().InitializeForBackup(0 as *mut u16);
			match hr {
				S_OK => {
					info!("[VSS] Initialized for Backup");
				},
				_ => {
					error!("[VSS] Couldn't Initialize for Backup");
					return Err(anyhow!("Couldn't Initialize for Backup"))
				}
			};

			// Setting Context
			hr = backup_components.as_ref().unwrap().SetContext(VSS_CTX_ALL as i32);
			match hr {
				S_OK => {
					info!("[VSS] Context Set");
				},
				_ => {
					error!("[VSS] Couldn't Set Context");
					return Err(anyhow!("Couldn't Set Context"))
				}
			};

			// Setting Backup State
			hr = backup_components.as_ref().unwrap().SetBackupState(
				true,
				true,
				VSS_BT_FULL,
				false
			);
			match hr {
				S_OK => {
					info!("[VSS] Backup State Set");
				},
				_ => {
					error!("[VSS] Couldn't Set Backup State");
					return Err(anyhow!("Couldn't Set Backup State"))
				}
			};

			// Querying for Snapshots
			hr = backup_components.as_ref().unwrap().Query(
				GUID_NULL, 
				VSS_OBJECT_NONE, 
				VSS_OBJECT_SNAPSHOT, 
				&mut enum_object
			);

			match hr {
				S_OK => {
					info!("[VSS] Snapshots Queried");
				},
				E_INVALIDARG => {
					error!("[VSS] Invalid argument");
					return Err(anyhow!("Invalid argument"))
				}
				_ => {
					error!("[VSS] Couldn't Query Snapshots");
					return Err(anyhow!("Couldn't Query Snapshots"))
				}
			}

			// Fetching Shadows

			let mut fetched: u32 = 0;

			loop {
				hr = enum_object.as_ref().unwrap().Next(
					1, 
					&mut prop, 
					&mut fetched
				); 

				match hr {
					S_OK => {
						if fetched == 0 {
							info!("[VSS] No more Shadow Copies!");
							break
						}
						// Snapshot Queried
						// Get Snapshot Info
						let snap: &mut VSS_SNAPSHOT_PROP = prop.Obj.Snap_mut();
			
						let mut prop_create: VSS_SNAPSHOT_PROP = Default::default();
						let hr : HRESULT = IVssBackupComponents::GetSnapshotProperties(&*backup_components,snap.m_SnapshotId, &mut prop_create);
						match hr {
							S_OK => {
								info!("[VSS] Snapshots path finded");
							},
							E_INVALIDARG => {
								println!("[VSS] Invalid argument");
								return Err(anyhow!("Invalid argument"))
							}
							_ => {
								error!("[VSS] Couldn't Query Snapshots");
								return Err(anyhow!("Couldn't Query Snapshots"))
							}
						}
						let out = U16CStr::from_ptr_str(prop_create.m_pwszSnapshotDeviceObject);
						info!("[VSS] VSS finded : {:?}",out.to_string_lossy());
						vss_list.push(out.to_string_lossy());

					},
					E_INVALIDARG => {
						error!("[VSS] Invalid argument");
						return Err(anyhow!("Invalid argument"));
					},
					_ => {
						info!("[VSS] No more Shadow Copies!");
						break
					}
				}
			}    

		Ok(vss_list)

		}

	}

}


impl Clone for Vss{
	fn clone(&self) -> Self {
		Vss{ dst: self.dst.clone(), artefacts_glob: self.artefacts_glob.clone() }
	}
}