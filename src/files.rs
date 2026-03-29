use std::{
    cmp::Reverse,
    fs::{self},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    process::Command,
    time::SystemTime,
};

const GO_BACK_FILE_NAME: &'static str = "..";

#[derive(Debug, Clone, Copy)]
pub enum SortBy {
    Name,
    Size,
    Modified,
}

#[derive(Debug, Clone, Copy)]
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

    fn sort_files_or_directories(
        records: &mut Vec<Self>,
        sort_by: SortBy,
        direction: SortDirection,
    ) {
        match direction {
            SortDirection::Ascending => match sort_by {
                SortBy::Name => records.sort_by(|a, b| {
                    let a_lower = a.file_name.to_lowercase();
                    let b_lower = b.file_name.to_lowercase();

                    a_lower.cmp(&b_lower)
                }),
                SortBy::Size => records.sort_by_key(|file| file.size),
                SortBy::Modified => records.sort_by_key(|file| file.modified),
            },
            SortDirection::Descending => match sort_by {
                SortBy::Name => records.sort_by(|a, b| {
                    let a_lower = a.file_name.to_lowercase();
                    let b_lower = b.file_name.to_lowercase();

                    b_lower.cmp(&a_lower)
                }),
                SortBy::Size => records.sort_by_key(|file| Reverse(file.size)),
                SortBy::Modified => {
                    records.sort_by_key(|file| Reverse(file.modified))
                }
            },
        }
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

pub fn sort_records(
    records: &mut Vec<FileRecord>,
    sort_by: SortBy,
    direction: SortDirection,
) {
    log::debug!("sort_records");
    log::debug!("records: {:#?}", records);

    let go_back_index = records.iter().position(|f| f.is_go_back_record);

    let go_back_record = match go_back_index {
        Some(idx) => Some(records.remove(idx)),
        None => None,
    };

    let mut files = vec![];
    let mut folders = vec![];

    loop {
        let option = records.pop();

        match option {
            Some(record) => {
                if record.is_file {
                    files.push(record);
                } else {
                    folders.push(record);
                }
            }
            None => break,
        }
    }

    log::debug!("after partion records: {:#?}", records);
    log::debug!("files: {:#?}", files);
    log::debug!("folders: {:#?}", folders);

    FileRecord::sort_files_or_directories(&mut files, sort_by, direction);
    FileRecord::sort_files_or_directories(&mut folders, sort_by, direction);

    if records.len() != 0 {
        panic!("records vector should be empty by now");
    }

    if let Some(record) = go_back_record {
        records.push(record);
    }

    records.append(&mut folders);
    records.append(&mut files);

    log::debug!("final records: {:#?}", records);
}

#[cfg(target_os = "macos")]
pub fn open_file(path_to_open: PathBuf) {
    log::debug!("open_file path_to_open: {}", path_to_open.display());

    let _ = Command::new("open").arg(path_to_open).spawn();
}
