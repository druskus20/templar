use anyhow::Result;
use std::path::PathBuf;

struct TemplarPaths {
    template_dir: PathBuf,
    output_dir: PathBuf,
    config: PathBuf,
}

impl TemplarPaths {
    fn try_from_env() -> Result<Self> {
        let template_dir = match std::env::var("TEMPLAR_TEMPLATE_DIR") {
            Ok(path) => PathBuf::from(path),
            Err(_) => std::env::current_dir()?,
        };

        let output_dir = match std::env::var("TEMPLAR_OUTPUT_DIR") {
            Ok(path) => PathBuf::from(path),
            Err(_) => PathBuf::from(std::env::var("HOME")?),
        };

        let config = match std::env::var("TEMPLAR_CONFIG") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                let home = std::env::var("HOME")?;
                PathBuf::from(
                    std::env::var("XDG_CONFIG_HOME")
                        .unwrap_or_else(|_| format!("{}/.config", home)),
                )
            }
        };

        Ok(TemplarPaths {
            template_dir,
            output_dir,
            config,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_try_from_env() {
        std::env::set_var("TEMPLAR_TEMPLATE_DIR", "template_dir");
        std::env::set_var("TEMPLAR_OUTPUT_DIR", "output_dir");
        std::env::set_var("TEMPLAR_CONFIG", "config");

        let templar_paths = TemplarPaths::try_from_env().unwrap();
        assert_eq!(templar_paths.template_dir, PathBuf::from("template_dir"));
        assert_eq!(templar_paths.output_dir, PathBuf::from("output_dir"));
        assert_eq!(templar_paths.config, PathBuf::from("config"));
    }
}
