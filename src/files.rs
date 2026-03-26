use std::{
    fs::{self, canonicalize},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    time::SystemTime,
};

const GO_BACK_FILE_NAME: &'static str = "..";

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FileRecord {
    pub file_name: String,
    pub is_file: bool,
    pub size: u64,
    pub modified: SystemTime,
    pub is_go_back_record: bool,
}

impl FileRecord {
    fn new_go_back_record() -> FileRecord {
        FileRecord {
            file_name: String::from(GO_BACK_FILE_NAME),
            is_file: false,
            size: 0,
            modified: SystemTime::UNIX_EPOCH,
            is_go_back_record: true,
        }
    }
}

pub fn read_directory(path: &PathBuf) -> Vec<FileRecord> {
    let canon_path = canonicalize(path).unwrap();
    let entries = fs::read_dir(path);

    match entries {
        Err(error) => {
            log::error!(
                "error during directory: '{}' read: {:#?}",
                path.display(),
                error
            );

            Vec::new()
        }
        Ok(entries) => {
            let mut files = vec![];
            let mut read_files: Vec<FileRecord> = entries
                .filter_map(|e| match e {
                    Ok(entry) => {
                        let file_name = String::from(
                            entry.file_name().into_string().unwrap(),
                        );

                        // TODO: filtering should be optional
                        if file_name.starts_with(".") {
                            None
                        } else {
                            let metadata = entry.metadata().unwrap();
                            let is_file = metadata.is_file();
                            let size = metadata.size();
                            let modified = metadata.modified().unwrap();

                            Some(FileRecord {
                                file_name,
                                is_file,
                                size,
                                modified,
                                is_go_back_record: false,
                            })
                        }
                    }
                    Err(error) => {
                        log::error!("{:#?}", error);
                        None
                    }
                })
                .collect();

            read_files.sort_by(|a, b| a.file_name.cmp(&b.file_name));

            log::debug!("path: {}", canon_path.display());

            if canon_path != PathBuf::from("/") {
                files.push(FileRecord::new_go_back_record());
            }

            files.append(&mut read_files);

            files
        }
    }
}

pub fn is_dir(path: &PathBuf) -> bool {
    let result = fs::metadata(path);

    match result {
        Ok(metadata) => metadata.is_dir(),
        Err(error) => {
            log::error!(
                "is_dir error - path: '{}', error: {:#?}",
                path.display(),
                error
            );

            false
        }
    }
}
