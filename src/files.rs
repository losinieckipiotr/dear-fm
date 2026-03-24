use std::{fs, path::PathBuf};

pub fn read_directory(path: &PathBuf) -> Vec<String> {
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
            let files: Vec<String> = entries
                .filter_map(|e| match e {
                    Ok(entry) => {
                        let file_name = String::from(
                            entry.file_name().into_string().unwrap(),
                        );

                        if file_name.starts_with(".") {
                            None
                        } else {
                            Some(file_name)
                        }
                    }
                    Err(error) => {
                        log::error!("{:#?}", error);
                        None
                    }
                })
                .collect();

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
