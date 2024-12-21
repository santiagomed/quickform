use async_openai::{
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ResponseFormat,
    },
    Client,
};
mod schemas;
mod prompts;

use prompts::{SYSTEM_PROMPT, entities};
use quickform::{App, state::Data};

#[derive(Debug, serde::Serialize, Clone)]
pub struct GenerationContext {
    pub user_prompt: String,
}

// #[template("templates/express/src/models/modelv3.ts.jinja")]
pub async fn entities(context: Data<GenerationContext>) -> Vec<schemas::entity::Entity> {
    let client = Client::new();

    let context = context.clone_inner().await;

    let messages = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content(SYSTEM_PROMPT)
            .build()
            .unwrap()
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(entities::PROMPT.replace("{user_prompt}", &context.user_prompt))
            .build()
            .unwrap()
            .into(),
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .response_format(ResponseFormat::JsonObject)
        .messages(messages)
        .build()
        .unwrap();
    let response = client.chat().create(request).await.unwrap();
    let entities = response.choices[0]
        .message
        .content
        .clone()
        .unwrap_or_default();

    println!("Entities: {}", entities);

    let entities: Vec<schemas::entity::Entity> = serde_json::from_str(&entities).unwrap();
    entities
}

#[tokio::main]
async fn main() {
    let cwd = std::env::current_dir().unwrap();
    let app = App::from_dir(cwd.join("../../templates/express"))
    .with_state(GenerationContext {
        user_prompt: "I need an e-commerce platform for selling electronics".to_string(),
    })
    .render_operation("entities.jinja", entities);

    let output_dir = std::path::Path::new("output");
    app.run(output_dir).await.unwrap();
}
