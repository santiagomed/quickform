use crate::generator::prompts::{entities, SYSTEM_PROMPT};
use crate::generator::schema;
use crate::generator::{GenerationContext, GenerationError};
use async_openai::{
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ResponseFormat,
    },
    Client,
};
use boil_codegen::template;

#[template("templates/express/src/models/modelv3.ts.jinja")]
pub async fn entities(context: &mut GenerationContext) -> Result<String, GenerationError> {
    let client = Client::new();

    let messages = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content(SYSTEM_PROMPT)
            .build()?
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(entities::PROMPT.replace("{user_prompt}", &context.user_prompt))
            .build()?
            .into(),
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .response_format(ResponseFormat::JsonObject)
        .messages(messages)
        .build()?;

    let response = client.chat().create(request).await?;
    let entities = response.choices[0]
        .message
        .content
        .clone()
        .unwrap_or_default();

    println!("Entities: {}", entities);

    let entities: Vec<schema::entity::Entity> = serde_json::from_str(&entities)?;
    entities
}

fn main() {
    println!("Hello, world!");
}
