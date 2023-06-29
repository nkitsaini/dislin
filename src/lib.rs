mod linear;
use linear::*;
use serde::Deserialize;
use worker::{wasm_bindgen::JsValue, *};

pub fn get_discord_webhook(ctx: &RouteContext<()>) -> Result<String> {
    let env = "DISCORD_WEBHOOK_URL";
    Ok(ctx.var(env).or_else(|_x| ctx.secret(env))?.to_string())
}

pub fn get_linear_api_key(ctx: &RouteContext<()>) -> Result<String> {
    let env = "LINEAR_API_KEY";
    Ok(ctx.var(env).or_else(|_x| ctx.secret(env))?.to_string())
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
struct LinearComment {
    id: String,
    body: String,
    #[serde(rename = "userId")]
    user_id: String,
    #[serde(rename = "issueId")]
    issue_id: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
enum LinearEvent {
    // TODO: Fill the rest from: https://developers.linear.app/docs/graphql/webhooks#data-change-events-payload
    Comment(LinearComment),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum LinearAction {
    Create,
    Update,
    Remove,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct LinearPayloadBody {
    action: LinearAction,

    #[serde(rename = "createdAt")]
    created_at: String,

    url: String,

    #[serde(flatten)]
    event: LinearEvent,
}

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let router = Router::new();

    worker::console_log!("Request ");
    router
        .get("/", |_req, _ctx| {
            return Response::ok("Yes, we're good.");
        })
        .post_async("/linear_webhook", |mut req, ctx| async move {
            let data: LinearPayloadBody = req.json().await?;
            console_log!("Linear Data: {:?}", &data);
            if data.action != LinearAction::Create {
                return Response::ok("Okay");
            }
            let comment = match data.event {
                LinearEvent::Comment(x) => x,
            };
            let webhook = get_discord_webhook(&ctx)?;
            let mut discord_msg = comment.body;
            let comment_info =
                fetch_comment_meta(&comment.id, &comment.user_id, &get_linear_api_key(&ctx)?)
                    .await?;
            discord_msg += "\n";
            discord_msg = format!(
                "[{}: {}]\n{}",
                &comment_info.issue_id, &comment_info.issue_title, discord_msg
            );

            let data =
                JsValue::from_str(&serde_json::json!({ "content": discord_msg, "username": &comment_info.creater_name }).to_string());
            let mut req_init = worker::RequestInit::new();
            req_init.with_method(Method::Post);
            req_init.with_body(Some(data));
            let mut request = worker::Request::new_with_init(&webhook, &req_init)?;
            request
                .headers_mut()?
                .set("content-type", "application/json")?;
            let mut resp = worker::Fetch::Request(request).send().await?;
            if resp.status_code() / 10 != 20 {
                worker::console_log!(
                    "Discord webhook request failed: status={}, content={:?}",
                    resp.status_code(),
                    resp.text().await
                );
                return Err("Invalid response code")?;
            }
            Response::ok("Okay")
        })
        .run(req, env)
        .await
}
