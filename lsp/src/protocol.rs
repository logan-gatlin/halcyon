use lsp_server::{
    Connection,
    ErrorCode,
    Message,
    Notification,
    Request,
    RequestId,
    Response,
    ResponseError,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn parse_request<T>(request: Request) -> Result<(RequestId, T), Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    Ok((request.id, serde_json::from_value(request.params)?))
}

pub fn parse_notification<T>(notification: Notification) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_value(notification.params)?)
}

pub fn send_response<T>(
    id: RequestId,
    result: Result<Option<T>, ResponseError>,
    connection: &Connection,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Serialize,
{
    let response = match result {
        Ok(result) => {
            Response {
                id,
                result: Some(serde_json::to_value(result)?),
                error: None,
            }
        }
        Err(error) => {
            Response {
                id,
                result: None,
                error: Some(error),
            }
        }
    };
    connection.sender.send(Message::Response(response))?;
    Ok(())
}

pub fn response_error(message: impl Into<String>) -> ResponseError {
    ResponseError {
        code: ErrorCode::InvalidParams as i32,
        message: message.into(),
        data: None,
    }
}
