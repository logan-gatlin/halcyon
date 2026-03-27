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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct DemoParams {
        value: String,
    }

    #[test]
    fn parse_request_deserializes_params() {
        let request = Request {
            id: RequestId::from(1),
            method: "demo/request".to_string(),
            params: json!({ "value": "ok" }),
        };

        let (id, params) =
            parse_request::<DemoParams>(request).expect("request params should deserialize");
        assert_eq!(id, RequestId::from(1));
        assert_eq!(params.value, "ok");
    }

    #[test]
    fn parse_request_rejects_invalid_param_shape() {
        let request = Request {
            id: RequestId::from(2),
            method: "demo/request".to_string(),
            params: json!("not-an-object"),
        };

        let parsed = parse_request::<DemoParams>(request);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_notification_deserializes_params() {
        let notification = Notification::new("demo/notify".to_string(), json!({ "value": "ok" }));

        let params = parse_notification::<DemoParams>(notification)
            .expect("notification params should deserialize");
        assert_eq!(params.value, "ok");
    }

    #[test]
    fn send_response_serializes_error_payload() {
        let (server_connection, client_connection) = Connection::memory();

        send_response::<serde_json::Value>(
            RequestId::from(3),
            Err(response_error("broken")),
            &server_connection,
        )
        .expect("error response should send");

        let message = client_connection
            .receiver
            .try_recv()
            .expect("response should be queued");
        let Message::Response(response) = message else {
            panic!("expected response message");
        };

        let error = response.error.expect("error payload should be present");
        assert_eq!(error.code, ErrorCode::InvalidParams as i32);
        assert_eq!(error.message, "broken");
    }

    #[test]
    fn response_error_uses_invalid_params_code() {
        let error = response_error("bad params");
        assert_eq!(error.code, ErrorCode::InvalidParams as i32);
        assert_eq!(error.message, "bad params");
    }
}
