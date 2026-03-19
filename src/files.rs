use std::{fs, io};

pub fn read_directory(path: &str) -> io::Result<Vec<String>> {
    let entries = fs::read_dir(path)?;

    let files: Vec<String> = entries
        .filter_map(|e| match e {
            Ok(entry) => {
                let file_name = String::from(entry.file_name().into_string().unwrap());

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

    Ok(files)

    // files.iter().for_each(|file| {
    //     log::info!("file: {file}");
    // });

    // Ok(())
}
