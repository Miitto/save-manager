use dioxus::prelude::*;

pub struct FileData(pub std::path::PathBuf);

impl dioxus::html::NativeFileData for FileData {
    fn name(&self) -> String {
        self.0.file_name().unwrap().to_string_lossy().into_owned()
    }

    fn size(&self) -> u64 {
        std::fs::metadata(&self.0).map(|m| m.len()).unwrap_or(0)
    }

    fn last_modified(&self) -> u64 {
        std::fs::metadata(&self.0)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }

    fn read_bytes(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<bytes::Bytes, dioxus_core::CapturedError>>
                + 'static,
        >,
    > {
        let path = self.0.clone();
        Box::pin(async move { Ok(bytes::Bytes::from(std::fs::read(&path)?)) })
    }

    fn read_string(
        &self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, dioxus_core::CapturedError>> + 'static>,
    > {
        let path = self.0.clone();
        Box::pin(async move { Ok(std::fs::read_to_string(&path)?) })
    }

    fn inner(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn path(&self) -> std::path::PathBuf {
        self.0.clone()
    }

    fn byte_stream(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn futures_util::Stream<Item = Result<bytes::Bytes, dioxus_core::CapturedError>>
                + 'static
                + Send,
        >,
    > {
        let path = self.0.clone();
        Box::pin(futures_util::stream::once(async move {
            Ok(bytes::Bytes::from(std::fs::read(&path)?))
        }))
    }

    fn content_type(&self) -> Option<String> {
        Some(
            dioxus::asset_resolver::native::get_mime_from_ext(
                self.0.extension().and_then(|ext| ext.to_str()),
            )
            .to_string(),
        )
    }
}
