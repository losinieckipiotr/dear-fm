use std::{
    cmp::Reverse,
    fs::{self, canonicalize},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    time::SystemTime,
};

const GO_BACK_FILE_NAME: &'static str = "..";

pub enum SortBy {
    Name,
    Size,
    Modified,
}

pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct FileRecord {
    pub file_name: String,
    pub is_file: bool,
    pub size: u64,
    pub modified: SystemTime,
    pub is_go_back_record: bool,
}

impl FileRecord {
    pub fn new_go_back_record() -> FileRecord {
        FileRecord {
            file_name: String::from(GO_BACK_FILE_NAME),
            is_file: false,
            size: 0,
            modified: SystemTime::UNIX_EPOCH,
            is_go_back_record: true,
        }
    }

    pub fn sort_records(
        files: &mut Vec<Self>,
        sort_by: SortBy,
        direction: SortDirection,
    ) {
        let go_back_index =
            files.iter().position(|f| f.is_go_back_record).unwrap();

        files.remove(go_back_index);

        match direction {
            SortDirection::Ascending => match sort_by {
                SortBy::Name => {
                    files.sort_by(|a, b| a.file_name.cmp(&b.file_name))
                }
                SortBy::Size => files.sort_by_key(|file| file.size),
                SortBy::Modified => files.sort_by_key(|file| file.modified),
            },
            SortDirection::Descending => match sort_by {
                SortBy::Name => {
                    files.sort_by_key(|file| Reverse(file.file_name.clone()))
                }
                SortBy::Size => files.sort_by_key(|file| Reverse(file.size)),
                SortBy::Modified => {
                    files.sort_by_key(|file| Reverse(file.modified))
                }
            },
        }

        files.insert(0, FileRecord::new_go_back_record());
    }
}

pub fn read_directory(path: &PathBuf) -> Vec<FileRecord> {
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
            let read_files: Vec<FileRecord> = entries
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

            read_files
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
