use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SubIntent {
    pub id: Uuid,
    pub text: String,
    pub retrieved_evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MultiIntentQuestion {
    pub id: Uuid,
    pub text: String,
    pub domain: String,
    pub intents: Vec<SubIntent>,
}
