use graphql_client::GraphQLQuery;
use serde::Deserialize;
use serde::Serialize;

use worker::wasm_bindgen::JsValue;
use worker::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql_schemas/linear_schema.json",
    query_path = "graphql_schemas/comment_info_query.graphql"
)]
pub struct CommentInfo;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct LinearComment {
    pub id: String,
    pub body: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "issueId")]
    pub issue_id: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum LinearEvent {
    // TODO: Fill the rest from: https://developers.linear.app/docs/graphql/webhooks#data-change-events-payload
    Comment(LinearComment),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LinearAction {
    Create,
    Update,
    Remove,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct LinearPayloadBody {
    pub action: LinearAction,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    pub url: Option<String>,

    #[serde(flatten)]
    pub event: LinearEvent,
}

pub struct LinearCommentMeta {
    pub creater_name: String,
    pub issue_title: String,
    pub issue_id: String,
}
const LINEAR_API: &'static str = "https://api.linear.app/graphql";

pub async fn fetch_comment_meta(
    comment_id: &str,
    user_id: &str,
    api_key: &str,
) -> Result<LinearCommentMeta> {
    let variables: comment_info::Variables = comment_info::Variables {
        comment_id: comment_id.to_string(),
        user_id: user_id.to_string(),
    };
    let mut writer = vec![];
    let mut serializer = serde_json::Serializer::new(&mut writer);

    let body: graphql_client::QueryBody<comment_info::Variables> =
        CommentInfo::build_query(variables);
    body.serialize(&mut serializer)?;
    let body = String::from_utf8(writer).unwrap();

    let data = JsValue::from_str(&body);
    let mut req_init = worker::RequestInit::new();
    req_init.with_method(Method::Post);
    req_init.with_body(Some(data));
    let mut request = worker::Request::new_with_init(LINEAR_API, &req_init)?;
    request
        .headers_mut()?
        .set("content-type", "application/json")?;
    request.headers_mut()?.set("Authorization", api_key)?;
    let mut resp = worker::Fetch::Request(request).send().await?;
    if resp.status_code() / 10 != 20 {
        worker::console_log!(
            "Linear request failed: status={}, content={:?}",
            resp.status_code(),
            resp.text().await
        );
        return Err("Invalid response code")?;
    }
    let body: graphql_client::Response<comment_info::ResponseData> = resp.json().await?;
    Ok(match body.data {
        Some(x) => LinearCommentMeta {
            creater_name: x.user.name,
            issue_title: x.comment.issue.title,
            issue_id: x.comment.issue.identifier,
        },
        None => {
            console_error!("Error: {:?}", body.errors);
            return Err("graphql error")?;
        }
    })
}

#[test]
fn payload_deser() {
    let data = r#"
{
  "action": "create",
  "data": {
    "id": "2174add1-f7c8-44e3-bbf3-2d60b5ea8bc9",
    "createdAt": "2020-01-23T12:53:18.084Z",
    "updatedAt": "2020-01-23T12:53:18.084Z",
    "archivedAt": null,
    "body": "Indeed, I think this is definitely an improvement over the previous version.",
    "edited": false,
    "issueId": "539068e2-ae88-4d09-bd75-22eb4a59612f",
    "userId": "aacdca22-6266-4c0a-ab3c-8fa70a26765c"
  },
  "type": "Comment",
  "url": "https://linear.app/issue/LIN-1778/foo-bar#comment-77217de3-fb52-4dad-bb9a-b356beb93de8",
  "createdAt": "2020-01-23T12:53:18.084Z",
  "webhookTimestamp": 1676056940508
}
"#;
    let data: LinearPayloadBody = serde_json::from_str(data).unwrap();
    let expected = LinearPayloadBody {
        action: LinearAction::Create,
        url:
            Some("https://linear.app/issue/LIN-1778/foo-bar#comment-77217de3-fb52-4dad-bb9a-b356beb93de8"
                .to_string()),
        created_at: "2020-01-23T12:53:18.084Z".to_string(),
        event: LinearEvent::Comment(LinearComment {
            id: "2174add1-f7c8-44e3-bbf3-2d60b5ea8bc9".to_string(),
            body: "Indeed, I think this is definitely an improvement over the previous version."
                .to_string(),
            user_id: "aacdca22-6266-4c0a-ab3c-8fa70a26765c".to_string(),
            issue_id: "539068e2-ae88-4d09-bd75-22eb4a59612f".to_string(),
        }),
    };
    assert_eq!(data, expected);
}
#[test]
fn payload_deser2() {
    let data = r#"
{
  "action": "create",
  "data": {
    "id": "2174add1-f7c8-44e3-bbf3-2d60b5ea8bc9",
    "createdAt": "2020-01-23T12:53:18.084Z",
    "updatedAt": "2020-01-23T12:53:18.084Z",
    "archivedAt": null,
    "body": "Indeed, I think this is definitely an improvement over the previous version.",
    "edited": false,
    "issueId": "539068e2-ae88-4d09-bd75-22eb4a59612f",
    "userId": "aacdca22-6266-4c0a-ab3c-8fa70a26765c"
  },
  "type": "Comment",
  "createdAt": "2020-01-23T12:53:18.084Z",
  "webhookTimestamp": 1676056940508
}
"#;
    let data: LinearPayloadBody = serde_json::from_str(data).unwrap();
    let expected = LinearPayloadBody {
        action: LinearAction::Create,
        url: None,
        created_at: "2020-01-23T12:53:18.084Z".to_string(),
        event: LinearEvent::Comment(LinearComment {
            id: "2174add1-f7c8-44e3-bbf3-2d60b5ea8bc9".to_string(),
            body: "Indeed, I think this is definitely an improvement over the previous version."
                .to_string(),
            user_id: "aacdca22-6266-4c0a-ab3c-8fa70a26765c".to_string(),
            issue_id: "539068e2-ae88-4d09-bd75-22eb4a59612f".to_string(),
        }),
    };
    assert_eq!(data, expected);
}
