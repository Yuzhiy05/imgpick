use std::path::{Path, PathBuf};
use std::fs;

pub fn get_image_files(dir: &Path) -> Vec<PathBuf> {
    let mut images = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_image_file(&path) {
                images.push(path);
            }
        }
    }
    
    images
}

pub fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "tif")
    } else {
        false
    }
}

pub fn copy_image_to_dir(src: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    if !src.exists() {
        return Err(format!("Source file not found: {}", src.display()));
    }
    
    if !dest_dir.exists() {
        fs::create_dir_all(dest_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    let file_name = src.file_name()
        .ok_or_else(|| "Invalid file name".to_string())?;
    let dest_path = dest_dir.join(file_name);
    
    fs::copy(src, &dest_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;
    
    Ok(dest_path)
}

pub fn create_directory_structure(base_dir: &Path, plan_name: &str) -> Result<PathBuf, String> {
    let plan_dir = base_dir.join(plan_name);
    let dirs = ["src", "pend", "priced", "proc"];
    
    for dir in dirs.iter() {
        let dir_path = plan_dir.join(dir);
        fs::create_dir_all(&dir_path)
            .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
    }
    
    Ok(plan_dir)
}

pub fn get_image_dimensions(path: &Path) -> Result<(u32, u32), String> {
    image::image_dimensions(path)
        .map_err(|e| format!("Failed to get image dimensions: {}", e))
}

pub fn generate_unique_filename(dir: &Path, original_name: &str) -> PathBuf {
    let path = dir.join(original_name);
    if !path.exists() {
        return path;
    }
    
    let stem = Path::new(original_name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let ext = Path::new(original_name)
        .extension()
        .unwrap_or_default()
        .to_string_lossy();
    
    for i in 1.. {
        let new_name = if ext.is_empty() {
            format!("{}_{}", stem, i)
        } else {
            format!("{}_{}.{}", stem, i, ext)
        };
        let new_path = dir.join(&new_name);
        if !new_path.exists() {
            return new_path;
        }
    }
    
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(Path::new("test.jpg")));
        assert!(is_image_file(Path::new("test.PNG")));
        assert!(is_image_file(Path::new("test.jpeg")));
        assert!(!is_image_file(Path::new("test.txt")));
        assert!(!is_image_file(Path::new("test")));
    }

    #[test]
    fn test_get_image_files() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();
        
        // Create test files
        fs::write(dir_path.join("test1.jpg"), b"test").unwrap();
        fs::write(dir_path.join("test2.png"), b"test").unwrap();
        fs::write(dir_path.join("test.txt"), b"test").unwrap();
        
        let images = get_image_files(dir_path);
        assert_eq!(images.len(), 2);
    }

    #[test]
    fn test_copy_image_to_dir() {
        let src_dir = tempdir().unwrap();
        let dest_dir = tempdir().unwrap();
        
        let src_file = src_dir.path().join("test.jpg");
        fs::write(&src_file, b"test image content").unwrap();
        
        let result = copy_image_to_dir(&src_file, dest_dir.path());
        assert!(result.is_ok());
        
        let dest_file = dest_dir.path().join("test.jpg");
        assert!(dest_file.exists());
    }

    #[test]
    fn test_create_directory_structure() {
        let base_dir = tempdir().unwrap();
        let result = create_directory_structure(base_dir.path(), "test_plan");
        assert!(result.is_ok());
        
        let plan_dir = result.unwrap();
        assert!(plan_dir.join("src").exists());
        assert!(plan_dir.join("pend").exists());
        assert!(plan_dir.join("priced").exists());
        assert!(plan_dir.join("proc").exists());
    }

    #[test]
    fn test_generate_unique_filename() {
        let dir = tempdir().unwrap();
        
        let path1 = generate_unique_filename(dir.path(), "test.jpg");
        assert_eq!(path1, dir.path().join("test.jpg"));
        
        // Create the file
        fs::write(&path1, b"test").unwrap();
        
        let path2 = generate_unique_filename(dir.path(), "test.jpg");
        assert_eq!(path2, dir.path().join("test_1.jpg"));
        
        fs::write(&path2, b"test").unwrap();
        
        let path3 = generate_unique_filename(dir.path(), "test.jpg");
        assert_eq!(path3, dir.path().join("test_2.jpg"));
    }
}
