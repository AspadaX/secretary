pub const OPENAI_API_BASE: &str = "https://api.openai.com/v1";
pub const OPENAI_CHAT_COMPLETION_ROUTE: &str = "/chat/completions";

pub const AZURE_OPENAI_COMPLETION_ROUTE: &str =
    "{endpoint}/openai/deployments/{deployment_id}/chat/completions?api-version={api_version}";
pub const AZURE_OPENAI_ENDPOINT_MARKER: &str = "{endpoint}";
pub const AZURE_OPENAI_DEPLOYMENT_ID_MARKER: &str = "{deployment_id}";
pub const AZURE_OPENAI_API_VERSION_MARKER: &str = "{api_version}";
