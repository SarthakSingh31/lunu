use lunu::{
    dotenvy::dotenv,
    email::{mail_server::MailServer, Email, Empty},
    Microservice, MICROSERVICE_ADDRS,
};
use tonic::transport::Server;

struct Mail {}

#[tonic::async_trait]
impl lunu::email::mail_server::Mail for Mail {
    async fn send(
        &self,
        request: tonic::Request<Email>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let Email {
            email,
            subject,
            body_html,
        } = request.into_inner();

        println!("Dummy sent email to {email}.\nSubject: {subject}\n{body_html}");

        Ok(tonic::Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let addr = MICROSERVICE_ADDRS[&Microservice::Email].parse()?;
    Server::builder()
        .add_service(MailServer::new(Mail {}))
        .serve(addr)
        .await?;

    Ok(())
}
