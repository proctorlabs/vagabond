macro_rules! config_file {
    ($class:ident ($file:literal) => $file_path:literal) => {
        mod t {
            #![allow(unused_must_use)]
            #[derive(Deref, Clone, Template)]
            #[template(path = $file)]
            pub struct $class(pub crate::VagabondConfig);
        }
        pub use t::$class;
        #[allow(dead_code)]
        impl $class {
            const CONFIG_NOTICE: &'static str =
                "This file is generated by Vagabond. Any changes may be overwritten!";

            const FILE_PATH: &'static str = $file_path;

            pub async fn write(config: crate::VagabondConfig) -> anyhow::Result<()> {
                use tokio::io::AsyncWriteExt;
                let config_contents = $class(config).to_string();
                let mut f = tokio::fs::File::create(Self::FILE_PATH).await?;
                f.write_all(config_contents.as_bytes()).await?;
                Ok(())
            }
        }
    };
}