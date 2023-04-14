use std::{env, path::PathBuf};

use lunu::{
    dotenvy::dotenv,
    storage::{self, storage_server::StorageServer, Exists, File, FileData, FileId},
    Microservice, MICROSERVICE_ADDRS,
};
use tokio::fs;
use tonic::transport::Server;

struct Storage {
    base_dir: PathBuf,
}

impl Storage {
    fn as_dir(&self, id: &FileId) -> PathBuf {
        let mut path = self.base_dir.clone();
        path.push(id.account_id.to_string());

        path
    }

    fn as_file_path(&self, id: &FileId) -> PathBuf {
        let mut path = self.as_dir(id);
        path.push(&id.name);

        path
    }
}

#[tonic::async_trait]
impl storage::storage_server::Storage for Storage {
    async fn put(
        &self,
        request: tonic::Request<File>,
    ) -> Result<tonic::Response<FileData>, tonic::Status> {
        let file = request.into_inner();
        let Some(id) = &file.id else {
            return Err(tonic::Status::invalid_argument("Missing file id to put the file"));
        };

        let dir = self.as_dir(id);
        fs::create_dir_all(dir)
            .await
            .map_err(|io| tonic::Status::internal(io.to_string()))?;

        let path = self.as_file_path(id);
        let old_file_data = fs::read(&path).await.ok();

        fs::write(path, &file.data)
            .await
            .map_err(|err| tonic::Status::internal(err.to_string()))?;

        Ok(tonic::Response::new(FileData {
            data: old_file_data,
        }))
    }

    async fn get(
        &self,
        request: tonic::Request<FileId>,
    ) -> Result<tonic::Response<FileData>, tonic::Status> {
        let id = request.into_inner();
        let path = self.as_file_path(&id);

        Ok(tonic::Response::new(FileData {
            data: fs::read(&path).await.ok(),
        }))
    }

    async fn has_file(
        &self,
        request: tonic::Request<FileId>,
    ) -> Result<tonic::Response<Exists>, tonic::Status> {
        let id = request.into_inner();
        let path = self.as_file_path(&id);

        Ok(tonic::Response::new(Exists {
            exisits: fs::try_exists(path).await.unwrap_or(false),
        }))
    }

    async fn delete(
        &self,
        request: tonic::Request<FileId>,
    ) -> Result<tonic::Response<FileData>, tonic::Status> {
        let id = request.into_inner();
        let path = self.as_file_path(&id);

        if fs::try_exists(&path).await.unwrap_or(false) {
            let data = fs::read(&path).await.ok();
            fs::remove_file(path)
                .await
                .map_err(|err| tonic::Status::internal(err.to_string()))?;

            Ok(tonic::Response::new(FileData { data }))
        } else {
            Ok(tonic::Response::new(FileData { data: None }))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("STORAGE_PATH").unwrap_or("./storage".to_string());

    let addr = MICROSERVICE_ADDRS[&Microservice::Storage].parse()?;
    Server::builder()
        .add_service(StorageServer::new(Storage {
            base_dir: database_url
                .parse()
                .expect("Failed to parse STORAGE_PATH as a path"),
        }))
        .serve(addr)
        .await?;

    Ok(())
}
