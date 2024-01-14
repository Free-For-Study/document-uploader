use anyhow::bail;
use drive::{
    hyper::{self, client::HttpConnector},
    hyper_rustls::{self, HttpsConnector},
    oauth2,
};
use futures::executor;
use google_drive3::oauth2::authenticator_delegate::{
    DefaultInstalledFlowDelegate, InstalledFlowDelegate,
};
use std::{fs, path::Path, str::FromStr};

use crate::{description::Description, empty_file::EmptyFile};

async fn browser_user_url(url: &str, need_code: bool) -> Result<String, String> {
    if open::that(url).is_ok() {
        println!("webbrowser was successfully opened.");
    }
    let def_delegate = DefaultInstalledFlowDelegate;
    def_delegate.present_user_url(url, need_code).await
}

#[derive(Copy, Clone)]
struct InstalledFlowBrowserDelegate;

impl InstalledFlowDelegate for InstalledFlowBrowserDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        need_code: bool,
    ) -> std::pin::Pin<Box<dyn futures::prelude::Future<Output = Result<String, String>> + Send + 'a>>
    {
        Box::pin(browser_user_url(url, need_code))
    }
}

pub struct DriveHub {
    instance: drive::DriveHub<HttpsConnector<HttpConnector>>,
}
impl DriveHub {
    pub async fn new(cache_folder: &Path) -> anyhow::Result<Self> {
        fs::create_dir_all(cache_folder)?;

        let secret: oauth2::ApplicationSecret =
            serde_json::from_reader(fs::File::open(cache_folder.join("secret.json"))?)?;

        let auth = oauth2::InstalledFlowAuthenticator::builder(
            secret,
            oauth2::InstalledFlowReturnMethod::HTTPPortRedirect(8001),
        )
        .persist_tokens_to_disk(cache_folder.join("auth"))
        .flow_delegate(Box::new(InstalledFlowBrowserDelegate))
        .build()
        .await?;

        let instance = drive::DriveHub::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            auth,
        );

        Ok(Self { instance })
    }

    pub async fn upload_document(&mut self, path: &Path) -> anyhow::Result<drive::api::File> {
        let description = fs::read_to_string(path.join("description.txt"))?;
        let description = Description::from_str(&description)?;

        let parent_folder = self
            .instance
            .files()
            .create(drive::api::File {
                name: Some(description.name.clone()),
                mime_type: Some("application/vnd.google-apps.folder".to_string()),
                ..Default::default()
            })
            .add_scope(google_drive3::api::Scope::Full)
            .upload(EmptyFile(), "application/vnd.google-apps.folder".parse()?)
            .await?
            .1;
        let parent_id = parent_folder.id.clone().unwrap();

        for entry in path.read_dir()? {
            let entry = entry?;
            let result = self
                .instance
                .files()
                .create(drive::api::File {
                    name: Some(description.name.clone()),
                    parents: Some(vec![parent_id.clone()]),
                    ..Default::default()
                })
                .add_scope(google_drive3::api::Scope::Full)
                .upload(
                    fs::File::open(&entry.path())?,
                    "application/octet-stream".parse()?,
                )
                .await?;
            if !result.0.status().is_success() {
                bail!("Failed to upload file: {:?}", entry.path());
            }
        }

        Ok(parent_folder)
    }

    pub fn upload_document_blocking(&mut self, path: &Path) -> anyhow::Result<drive::api::File> {
        executor::block_on(self.upload_document(path))
    }
}
