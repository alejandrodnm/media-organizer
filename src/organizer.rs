pub mod photos;
pub mod videos;
use crate::directory::FilesIter;
use color_eyre::eyre::{eyre, Result, WrapErr};
use std::fs;
use std::path::PathBuf;

pub trait MediaTypeOrganizer {
    fn should_organize(&self, item: &PathBuf) -> bool;
    fn destination_dir(&self, item: &PathBuf) -> Result<PathBuf>;
}

pub struct Organizer {
    media_type_organizers: Vec<Box<dyn MediaTypeOrganizer>>,
}

impl Organizer {
    pub fn new(media_type_organizers: Vec<Box<dyn MediaTypeOrganizer>>) -> Organizer {
        Organizer {
            media_type_organizers,
        }
    }

    pub fn organize(&self, media_src: PathBuf) -> Result<()> {
        for file in FilesIter::new(media_src) {
            for media_type_organizer in &self.media_type_organizers {
                if !media_type_organizer.should_organize(&file) {
                    continue;
                }
                let dst_dir = match media_type_organizer
                    .destination_dir(&file)
                    .wrap_err_with(|| format!("failed to get destination dir from {:?}", file))
                {
                    Ok(dir) => dir,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        continue;
                    }
                };

                if let Err(e) = Organizer::move_file(&file, &dst_dir).wrap_err_with(|| {
                    format!(
                        "failed to move file {:?} to destination dir {:?}",
                        file, dst_dir
                    )
                }) {
                    eprintln!("{:?}", e);
                }
            }
        }
        Ok(())
    }

    fn move_file(file: &PathBuf, dst_dir: &PathBuf) -> Result<()> {
        if !dst_dir.is_dir() {
            fs::create_dir_all(&dst_dir).wrap_err("failed to create destination dir")?;
        }

        let file_name = match file.file_name() {
            Some(name) => name,
            None => return Err(eyre!("failed to get file name")),
        };
        let dst_path = &dst_dir.join(file_name);
        if dst_path.is_file() {
            return Err(eyre!(
                "a file with the same name already exists in the destination path"
            ));
        }
        fs::rename(file, dst_path).wrap_err("failed to move file to destination dir")
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use photos::PhotoOrganizer;
    use tempfile::TempDir;
    use videos::VideoOrganizer;

    #[test]
    fn organize() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        let exif_photo = PathBuf::from(file!())
            .parent()
            .unwrap()
            .join("fixtures")
            .join("camera.jpg");
        fs::copy(exif_photo, src.path().join("camera.jpg")).unwrap();

        let wa_photo = PathBuf::from(file!())
            .parent()
            .unwrap()
            .join("fixtures")
            .join("IMG-20200407-WA0004.jpg");
        let sub_dir = src.path().join("sub_dir");
        fs::create_dir(&sub_dir).unwrap();
        fs::copy(wa_photo, sub_dir.join("IMG-20200407-WA0004.jpg")).unwrap();

        let video = PathBuf::from(file!())
            .parent()
            .unwrap()
            .join("fixtures")
            .join("20200829_205420.mp4");
        let sub_sub_dir = sub_dir.join("sub_dir");
        fs::create_dir(&sub_sub_dir).unwrap();
        fs::copy(video, sub_sub_dir.join("20200829_205420.mp4")).unwrap();

        Organizer::new(vec![
            Box::new(PhotoOrganizer::new(dst.path().to_path_buf())),
            Box::new(VideoOrganizer::new(dst.path().to_path_buf())),
        ])
        .organize(src.path().to_path_buf())
        .unwrap();

        assert!(dst
            .path()
            .join("2019")
            .join("01 - January")
            .join("camera.jpg")
            .is_file());

        assert!(dst
            .path()
            .join("2020")
            .join("04 - April")
            .join("IMG-20200407-WA0004.JPG")
            .is_file());

        assert!(dst
            .path()
            .join("2020")
            .join("20200829_205420.mp4")
            .is_file());
    }
}
